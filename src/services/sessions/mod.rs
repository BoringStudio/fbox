mod websocket;

use bip39::{Language, Mnemonic, MnemonicType, Seed};
use futures::StreamExt;
use uuid::Uuid;
use warp::ws::WebSocket;

use self::websocket::ConnectionId;
use self::websocket::Event;
use crate::prelude::*;

#[derive(Debug)]
struct Session {
    seed: Seed,
    connections: HashMap<ConnectionId, Arc<Connection>>,
    files: HashMap<Uuid, FileInfo>,
}

impl Session {
    pub fn new<T: AsRef<str>>(mnemonic: &Mnemonic, password: T, host: Arc<Connection>) -> Self {
        let seed = Seed::new(mnemonic, password.as_ref());

        let mut connections = HashMap::new();
        connections.insert(host.id(), host);

        Self {
            seed,
            connections,
            files: Default::default(),
        }
    }

    pub fn broadcast_external(&self, message: &WsResponse) {
        for (_, peer) in self.connections.iter() {
            peer.send_external(message);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileInfo {
    id: Uuid,
    name: String,
    size: usize,
}

pub struct SessionService {
    settings: Arc<Settings>,
    pending_connections: ArcRwLock<HashMap<Phrase, Arc<Connection>>>,
    sessions: ArcRwLock<HashMap<Seed, ArcRwLock<Session>>>,
}

impl SessionService {
    pub fn new(settings: Arc<Settings>) -> Self {
        Self {
            settings,
            pending_connections: Arc::new(Default::default()),
            sessions: Arc::new(Default::default()),
        }
    }

    pub async fn handle_connection(&self, websocket: WebSocket) {
        let (conn, mut rx) = websocket::init_connection(websocket);

        // add connection to pending
        let local_mnemonic = {
            // generate mnemonic
            let mut mnemonic = self.generate_mnemonic();

            let mut pending_connections = self.pending_connections.write().await;

            // prevent collisions
            loop {
                if pending_connections.contains_key(mnemonic.phrase()) {
                    mnemonic = self.generate_mnemonic();
                } else {
                    break;
                }
            }

            let phrase = mnemonic.phrase().to_owned();
            pending_connections.insert(phrase, conn.clone());

            mnemonic
        };

        conn.send_external(&WsResponse::Created {
            phrase: local_mnemonic.phrase().to_owned(),
        });

        log::debug!("create ws connection: {}", conn.id());

        let mut local_session: Option<ArcRwLock<Session>> = None;

        while let Some(request) = rx.next().await {
            log::trace!("id=[{:0>5}], received request: {:?}", conn.id(), request);

            match request {
                Event::External(WsRequest::Connect { phrase }) => {
                    const MIN_PHRASE_LEN: usize = 6 * 3 + 5; // 6 words with 3 letters + 5 spaces
                    const MAX_PHRASE_LEN: usize = 6 * 8 + 5; // 6 words with 8 letters + 5 spaces
                    if phrase.len() < MIN_PHRASE_LEN || phrase.len() > MAX_PHRASE_LEN || phrase == local_mnemonic.phrase() {
                        log::trace!("id=[{:0>5}], invalid phrase", conn.id());
                        conn.send_external(&WsResponse::PeerNotFound);
                        continue;
                    }

                    let peer = match self.remove_pending_peer(&phrase).await {
                        Some(entry) => entry,
                        None => {
                            log::trace!("id=[{:0>5}], peer not found", conn.id());
                            conn.send_external(&WsResponse::PeerNotFound);
                            continue;
                        }
                    };

                    // If session exists
                    if let Some(session) = local_session.as_ref() {
                        log::trace!("id=[{:0>5}], session exists", conn.id());
                        let (seed, files) = {
                            let mut session = session.write().await;

                            log::trace!("id=[{:0>5}], add peer to connections, peer id: {}", conn.id(), peer.id());

                            // Add peer to connections
                            session.connections.insert(peer.id(), peer.clone());
                            let seed = session.seed.clone();
                            let files = session.files.iter().map(|(_, file)| file).cloned().collect::<Vec<_>>();
                            (seed, files)
                        };
                        let seed = encode_seed(&seed);

                        peer.send_internal(InternalMessage::SessionCreated(session.clone()));
                        peer.send_external(&WsResponse::Connected { seed, files });
                    } else {
                        log::trace!("id=[{:0>5}], create new session", conn.id());

                        log::trace!("id=[{:0>5}], remove current connection from pending", conn.id());

                        // Remove host from pending peer
                        self.remove_pending_peer(local_mnemonic.phrase()).await;

                        // Create new session
                        let mut session = Session::new(&local_mnemonic, &self.settings.password, conn.clone());
                        let seed = session.seed.clone();
                        let encoded_seed = encode_seed(&seed);

                        // Add peer to connections
                        session.connections.insert(peer.id(), peer.clone());

                        let session = Arc::new(RwLock::new(session));

                        // Init local session
                        local_session = Some(session.clone());

                        log::trace!("id=[{:0>5}], save new session", conn.id());

                        // Add new session to self sessions
                        self.sessions.write().await.insert(seed, session.clone());

                        // Send messages
                        peer.send_internal(InternalMessage::SessionCreated(session.clone()));

                        let connection_message = WsResponse::Connected {
                            seed: encoded_seed.clone(),
                            files: Default::default(),
                        };
                        peer.send_external(&connection_message);
                        conn.send_external(&connection_message);
                    }
                }
                Event::External(WsRequest::AddFile { name, size }) => {
                    let mut session = match &local_session {
                        Some(session) => session.write().await,
                        None => {
                            conn.send_external(&WsResponse::SessionNotFound);
                            continue;
                        }
                    };

                    if session.files.len() + 1 >= MAX_FILE_COUNT {
                        conn.send_external(&WsResponse::FileCountLimitReached);
                        continue;
                    }

                    let file_info = FileInfo {
                        id: Uuid::new_v4(),
                        name,
                        size,
                    };
                    session.files.insert(file_info.id, file_info.clone());
                    session.broadcast_external(&WsResponse::FileAdded(file_info));
                }
                Event::External(WsRequest::RemoveFile { id }) => {
                    let mut session = match &local_session {
                        Some(session) => session.write().await,
                        None => {
                            conn.send_external(&WsResponse::SessionNotFound);
                            continue;
                        }
                    };

                    if session.files.remove(&id).is_some() {
                        session.broadcast_external(&WsResponse::FileRemoved { id });
                    }
                }
                Event::Internal(InternalMessage::SessionCreated(new_session)) => local_session = Some(new_session),
                Event::Internal(InternalMessage::PeerDisconnected(id)) => {
                    log::info!("peer with id {} disconnected, local id: {}", id, conn.id());
                }
            };
        }

        log::trace!("id=[{:0>5}], websocket connection closed", conn.id());

        // Remove host from pending
        match self.remove_pending_peer(local_mnemonic.phrase()).await {
            Some(_) => (),
            None => {
                // If session exists
                if let Some(session) = local_session {
                    log::trace!("id=[{:0>5}], session exists", conn.id());
                    let session_seed = {
                        log::trace!("id=[{:0>5}], remove current connection from session", conn.id());

                        // Remove self from session connections
                        let mut session = session.write().await;
                        session.connections.remove(&conn.id());

                        log::trace!("id=[{:0>5}], notify all peers", conn.id());
                        // Notify all peers
                        session.connections.iter().for_each(|(_, another_conn)| {
                            another_conn.send_internal(InternalMessage::PeerDisconnected(conn.id()));
                        });

                        if session.connections.is_empty() {
                            Some(session.seed.clone())
                        } else {
                            None
                        }
                    };

                    // Remove session from `self.sessions` if session is empty
                    if let Some(seed) = session_seed {
                        log::trace!("id=[{:0>5}], delete session", conn.id());
                        self.sessions.write().await.remove(&seed);
                    }
                } else {
                    log::warn!("peer disconnected without any sessions: {}", conn.id())
                    // do nothing
                }
            }
        }

        //
        log::debug!("drop ws connection: {}", conn.id());
    }

    pub fn generate_mnemonic(&self) -> Mnemonic {
        Mnemonic::new(MnemonicType::Words6, Language::English)
    }

    async fn remove_pending_peer<T: AsRef<str>>(&self, phrase: T) -> Option<Arc<Connection>> {
        let mut pending_connections = self.pending_connections.write().await;
        // Remove peer from `pending_connections`
        pending_connections.remove(phrase.as_ref())
    }
}

fn encode_seed(seed: &Seed) -> String {
    base64::encode_config(seed.as_bytes(), base64::Config::new(base64::CharacterSet::UrlSafe, true))
}

#[derive(Debug, Clone)]
enum InternalMessage {
    SessionCreated(ArcRwLock<Session>),
    PeerDisconnected(ConnectionId),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "content", rename_all = "snake_case")]
enum WsRequest {
    Connect { phrase: String },
    AddFile { name: String, size: usize },
    RemoveFile { id: Uuid },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "snake_case")]
enum WsResponse {
    Created { phrase: String },
    Connected { seed: String, files: Vec<FileInfo> },
    PeerNotFound,
    SessionNotFound,
    FileCountLimitReached,
    FileAdded(FileInfo),
    FileRemoved { id: Uuid },
}

const MAX_FILE_COUNT: usize = 10;

type Connection = websocket::Connection<InternalMessage, WsResponse>;
type Phrase = String;
