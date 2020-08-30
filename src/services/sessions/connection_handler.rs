use bip39::{Language, Mnemonic, MnemonicType};
use bytes::Buf;
use futures::{Stream, StreamExt};
use uuid::Uuid;
use warp::ws::WebSocket;

use super::session::{Connection, Session};
use super::websocket::ConnectionId;
use super::websocket::Event;
use super::SessionService;
use crate::prelude::*;

pub struct ConnectionHandler {
    session_service: Arc<SessionService>,
    conn: Arc<Connection>,
    mnemonic: Mnemonic,
    session: Option<ArcRwLock<Session>>,
}

impl ConnectionHandler {
    pub fn new(session_service: Arc<SessionService>, conn: Arc<Connection>) -> Self {}

    async fn connect(&self, peer_phrase: String) {}
}
