use bip39::Mnemonic;
use uuid::Uuid;

use super::websocket::{self, ConnectionId};
use crate::prelude::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "content", rename_all = "snake_case")]
pub enum WsRequest {
    Connect {
        phrase: String,
    },
    AddFile {
        id: Uuid,
        name: String,
        mime_type: String,
        size: usize,
    },
    RemoveFile {
        id: Uuid,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "snake_case")]
pub enum WsResponse {
    Created {
        phrase: String,
    },
    Connected {
        connection_id: usize,
        seed: String,
        files: Vec<FileInfo>,
    },
    FileAdded(FileInfo),
    FileRemoved {
        id: Uuid,
    },
    FileRequested {
        id: Uuid,
    },
    PeerNotFound,
    SessionNotFound,
    FileCountLimitReached,
    FileAlreadyExists,
}

#[derive(Debug, Clone)]
pub enum InternalMessage {
    SessionCreated(ArcRwLock<Session>),
}

#[derive(Debug)]
pub struct Session {
    pub seed: Seed,
    pub connections: HashMap<ConnectionId, Arc<Connection>>,
    pub files: HashMap<Uuid, FileInfo>,
    pub pending_requests: HashMap<Uuid, mpsc::UnboundedSender<bytes::Bytes>>,
}

impl Session {
    pub fn new<T: AsRef<str>>(mnemonic: &Mnemonic, password: T, host: Arc<Connection>) -> Self {
        let seed = bip39::Seed::new(mnemonic, password.as_ref());

        let mut connections = HashMap::new();
        connections.insert(host.id(), host);

        Self {
            seed: seed.into_bytes(),
            connections,
            files: Default::default(),
            pending_requests: Default::default(),
        }
    }

    pub fn broadcast_external(&self, message: &WsResponse) {
        self.connections.iter().for_each(|(_, peer)| peer.send_external(message))
    }

    pub fn broadcast_external_except(&self, connection_id: usize, message: &WsResponse) {
        self.connections
            .iter()
            .filter(|(&id, _)| id != connection_id)
            .for_each(|(_, peer)| peer.send_external(message));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: usize,
    pub connection_id: usize,
}

pub type Connection = websocket::Connection<InternalMessage, WsResponse>;
pub type Phrase = String;
pub type Seed = Vec<u8>;
