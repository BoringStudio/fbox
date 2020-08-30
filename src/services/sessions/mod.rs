mod session;
mod websocket;

use bip39::{Language, Mnemonic, MnemonicType};
use bytes::Buf;
use futures::{Stream, StreamExt};
use uuid::Uuid;
use warp::ws::WebSocket;

use self::session::*;
use self::websocket::Event;
use crate::prelude::*;

pub type PendingConnections = RwLock<HashMap<Phrase, Arc<Connection>>>;
pub type Sessions = RwLock<HashMap<Seed, ArcRwLock<Session>>>;

pub struct SessionService {
    seed_password: String,
    pending_connections: PendingConnections,
    sessions: Sessions,
}

impl SessionService {
    pub fn new(settings: &Settings) -> Arc<Self> {
        Arc::new(Self {
            seed_password: settings.password.clone(),
            pending_connections: Default::default(),
            sessions: Default::default(),
        })
    }

    pub async fn request_file(&self, id: Uuid, seed: String) -> Option<(FileInfo, mpsc::UnboundedReceiver<bytes::Bytes>)> {
        let seed = decode_seed(&seed).ok()?;
        log::debug!("decoded seed: {:?}", seed);

        let session = self.sessions.read().await.get(&seed)?.clone();
        log::debug!("found session");

        let mut session = session.write().await;
        let file = session.files.get(&id)?.clone();
        log::debug!("found file");

        let file_owner = session.connections.get(&file.connection_id)?.clone();
        log::debug!("found file owner");

        if session.pending_requests.contains_key(&id) {
            return None;
        }

        let (tx, rx) = mpsc::unbounded_channel();
        session.pending_requests.insert(id, tx);
        log::debug!("created request");

        file_owner.send_external(&WsResponse::FileRequested { id });

        Some((file, rx))
    }

    pub async fn upload_file<T, I>(&self, id: Uuid, seed: String, data: T) -> Option<()>
    where
        T: Stream<Item = Result<I, warp::Error>>,
        I: Buf,
    {
        let seed = decode_seed(&seed).ok()?;
        log::debug!("decoded seed: {:?}", seed);

        let stream = {
            let session = self.sessions.read().await.get(&seed)?.clone();
            let mut session = session.write().await;
            session.pending_requests.remove(&id)?
        };

        data.for_each(|part| async {
            match part {
                Ok(mut part) => {
                    println!("part: {:?}", part.remaining());
                    stream
                        .send(part.to_bytes())
                        .unwrap_or_else(|e| println!("error: {}", e.to_string()));
                }
                Err(e) => {
                    println!("error: {:?}", e);
                }
            }
        })
        .await;

        Some(())
    }

    pub async fn handle_connection(&self, websocket: WebSocket) {
        let (conn, mut rx) = websocket::init_connection(websocket);

        // add connection to pending
        let local_mnemonic = self.create_pending_connection(conn.clone()).await;
        let mut local_session: Option<ArcRwLock<Session>> = None;

        // notify created
        conn.send_external(&WsResponse::Created {
            phrase: local_mnemonic.phrase().to_owned(),
        });

        // handle events
        while let Some(request) = rx.next().await {
            match request {
                Event::External(WsRequest::Connect { phrase }) => {
                    const MIN_PHRASE_LEN: usize = 6 * 3 + 5; // 6 words with 3 letters + 5 spaces
                    const MAX_PHRASE_LEN: usize = 6 * 8 + 5; // 6 words with 8 letters + 5 spaces
                    if phrase.len() < MIN_PHRASE_LEN || phrase.len() > MAX_PHRASE_LEN || phrase == local_mnemonic.phrase() {
                        conn.send_external(&WsResponse::PeerNotFound);
                        continue;
                    }

                    let peer = match self.remove_pending_peer(&phrase).await {
                        Some(entry) => entry,
                        None => {
                            conn.send_external(&WsResponse::PeerNotFound);
                            continue;
                        }
                    };

                    // If session exists
                    if let Some(session) = local_session.as_ref() {
                        let (seed, files) = {
                            let mut session = session.write().await;

                            // Add peer to connections
                            session.connections.insert(peer.id(), peer.clone());
                            let seed = session.seed.clone();
                            let files = session.files.iter().map(|(_, file)| file).cloned().collect::<Vec<_>>();
                            (seed, files)
                        };
                        let seed = encode_seed(&seed);

                        peer.send_internal(InternalMessage::SessionCreated(session.clone()));
                        peer.send_external(&WsResponse::Connected {
                            connection_id: peer.id(),
                            seed,
                            files,
                        });
                    } else {
                        // Remove host from pending peer
                        self.remove_pending_peer(local_mnemonic.phrase()).await;

                        // Create new session
                        let mut session = Session::new(&local_mnemonic, &self.seed_password, conn.clone());
                        let seed = session.seed.clone();
                        let encoded_seed = encode_seed(&seed);

                        // Add peer to connections
                        session.connections.insert(peer.id(), peer.clone());

                        let session = Arc::new(RwLock::new(session));

                        // Init local session
                        local_session = Some(session.clone());

                        // Add new session to self sessions
                        self.sessions.write().await.insert(seed, session.clone());

                        // Send messages
                        peer.send_internal(InternalMessage::SessionCreated(session.clone()));

                        peer.send_external(&WsResponse::Connected {
                            connection_id: peer.id(),
                            seed: encoded_seed.clone(),
                            files: Default::default(),
                        });
                        conn.send_external(&WsResponse::Connected {
                            connection_id: conn.id(),
                            seed: encoded_seed,
                            files: Default::default(),
                        });
                    }
                }
                Event::External(WsRequest::AddFile { id, name, mime_type, size }) => {
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

                    if session.files.contains_key(&id) {
                        conn.send_external(&WsResponse::FileAlreadyExists);
                        continue;
                    }

                    let file_info = FileInfo {
                        id,
                        name,
                        mime_type,
                        size,
                        connection_id: conn.id(),
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
            };
        }

        self.remove_connection(conn, local_mnemonic, local_session).await
    }

    async fn remove_pending_peer<T: AsRef<str>>(&self, phrase: T) -> Option<Arc<Connection>> {
        let mut pending_connections = self.pending_connections.write().await;
        // Remove peer from `pending_connections`
        pending_connections.remove(phrase.as_ref())
    }

    async fn create_pending_connection(&self, conn: Arc<Connection>) -> Mnemonic {
        let mtype = MnemonicType::Words6;
        let lang = Language::English;

        // generate mnemonic
        let mut mnemonic = Mnemonic::new(mtype, lang);

        let mut pending_connections = self.pending_connections.write().await;

        // prevent collisions
        loop {
            if pending_connections.contains_key(mnemonic.phrase()) {
                mnemonic = Mnemonic::new(mtype, lang);
            } else {
                break;
            }
        }

        let phrase = mnemonic.phrase().to_owned();
        pending_connections.insert(phrase, conn);

        mnemonic
    }

    async fn remove_connection(&self, conn: Arc<Connection>, local_mnemonic: Mnemonic, local_session: Option<ArcRwLock<Session>>) {
        let _ = self.remove_pending_peer(local_mnemonic.phrase()).await;
        let session = match local_session {
            Some(session) => session,
            None => return,
        };

        let session_seed = {
            // Remove self from session connections
            let mut session = session.write().await;
            session.connections.remove(&conn.id());

            // Remove all owned files
            let conn_files = session
                .files
                .iter()
                .filter_map(|(_, file)| if file.connection_id == conn.id() { Some(file.id) } else { None })
                .collect::<Vec<_>>();
            for id in conn_files.into_iter() {
                session.files.remove(&id);
                session.broadcast_external_except(conn.id(), &WsResponse::FileRemoved { id });
            }

            if session.connections.is_empty() {
                Some(session.seed.clone())
            } else {
                None
            }
        };

        // Remove session from `self.sessions` if session is empty
        if let Some(seed) = session_seed {
            self.sessions.write().await.remove(&seed);
        }
    }
}

fn encode_seed(seed: &[u8]) -> String {
    base64::encode_config(seed, base64::Config::new(base64::CharacterSet::UrlSafe, true))
}

fn decode_seed(seed: &str) -> Result<Vec<u8>, base64::DecodeError> {
    base64::decode_config(seed, base64::Config::new(base64::CharacterSet::UrlSafe, true))
}

const MAX_FILE_COUNT: usize = 10;
