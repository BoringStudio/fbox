mod websocket;

use bip39::{Language, Mnemonic, MnemonicType, Seed};
use futures::StreamExt;
use warp::ws::WebSocket;

use self::websocket::Event;
use crate::prelude::*;

#[derive(Debug)]
pub struct Session {
    seed: Seed,
    connections: HashMap<usize, Arc<Connection>>,
}

pub struct SessionService {
    settings: Arc<Settings>,
    pending_connections: ArcRwLock<HashMap<String, (Arc<Connection>, Mnemonic)>>,
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
        let local_phrase = {
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
            pending_connections.insert(phrase.clone(), (conn.clone(), mnemonic));

            phrase
        };

        conn.send_external(&WsResponse::Created {
            phrase: local_phrase.clone(),
        });

        log::debug!("create ws connection: {}", conn.id());

        let mut session: Option<ArcRwLock<Session>> = None;

        while let Some(request) = rx.next().await {
            log::debug!("received request: {:?}", request);

            match request {
                Event::External(WsRequest::Connect { phrase }) => {
                    const MIN_PHRASE_LEN: usize = 6 * 3 + 5; // 6 words with 3 letters + 5 spaces
                    const MAX_PHRASE_LEN: usize = 6 * 8 + 5; // 6 words with 8 letters + 5 spaces
                    if phrase.len() < MIN_PHRASE_LEN
                        || phrase.len() > MAX_PHRASE_LEN
                        || phrase == local_phrase
                    {
                        conn.send_external(&WsResponse::PeerNotFound);
                        continue;
                    }

                    // find peer
                    let pending_connections = self.pending_connections.read().await;
                    let (_peer, mnemonics) = match pending_connections.get(&phrase) {
                        Some(entry) => entry,
                        None => {
                            conn.send_external(&WsResponse::PeerNotFound);
                            continue;
                        }
                    };

                    if let Some(session) = session.as_ref() {
                        let session = session.read().await;

                        conn.send_external(&WsResponse::Connected {
                            seed: encode_seed(&session.seed),
                        })
                    } else {
                        let seed = Seed::new(mnemonics, &self.settings.password);

                        conn.send_external(&WsResponse::Connected {
                            seed: encode_seed(&seed),
                        });
                    }
                }
                Event::Internal(InternalMessage::SessionCreated(new_session)) => {
                    session = Some(new_session)
                }
            };
        }

        // remove connection from pending
        let is_still_pending = self
            .pending_connections
            .read()
            .await
            .contains_key(&local_phrase);
        if is_still_pending {
            let mut pending_connections = self.pending_connections.write().await;
            pending_connections.remove(&local_phrase);
        }

        //
        log::debug!("drop ws connection: {}", conn.id());
    }

    pub fn generate_mnemonic(&self) -> Mnemonic {
        Mnemonic::new(MnemonicType::Words6, Language::English)
    }
}

fn encode_seed(seed: &Seed) -> String {
    base64::encode_config(
        seed.as_bytes(),
        base64::Config::new(base64::CharacterSet::UrlSafe, true),
    )
}

#[derive(Debug, Clone)]
enum InternalMessage {
    SessionCreated(ArcRwLock<Session>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "content")]
enum WsRequest {
    Connect { phrase: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "content")]
enum WsResponse {
    Created { phrase: String },
    Connected { seed: String },
    PeerNotFound,
}

type Connection = websocket::Connection<InternalMessage, WsResponse>;
