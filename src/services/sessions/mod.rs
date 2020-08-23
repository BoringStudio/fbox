use std::sync::atomic::{AtomicUsize, Ordering};

use bip39::{Language, Mnemonic, MnemonicType, Seed};
use futures::{FutureExt, StreamExt};
use warp::ws::{self, WebSocket};

use crate::prelude::*;

pub struct Session {}

pub struct SessionService {
    settings: Arc<Settings>,
    sessions: ArcRwLock<HashMap<Seed, ArcRwLock<Session>>>,
}

impl SessionService {
    pub fn new(settings: Arc<Settings>) -> Self {
        Self {
            settings,
            sessions: Arc::new(Default::default()),
        }
    }

    pub async fn handle_connection(&self, websocket: WebSocket) {
        let (conn, mut rx) = init_connection(websocket);

        while let Some(request) = rx.next().await {
            log::debug!("received request: {:?}", request);

            conn.tx.send(&WsResponse::OnCreated {
                phrase: self.generate_mnemonic().await.into_phrase(),
            });
        }
    }

    pub async fn generate_mnemonic(&self) -> Mnemonic {
        let kind = MnemonicType::Words6;
        let lang = Language::English;
        Mnemonic::new(kind, lang)
    }
}

fn init_connection(websocket: WebSocket) -> (Connection, ClientRx<WsRequest>) {
    let (ws_tx, ws_rx) = websocket.split();
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::task::spawn(rx.forward(ws_tx).map(|result| {
        if let Err(e) = result {
            log::trace!("websocket close error: {}", e);
        }
    }));

    let connection = Connection::new(tx.into());
    let client_rx = ClientRx::<WsRequest>::from(ws_rx);

    (connection, client_rx)
}

#[derive(Debug, Clone)]
struct Connection {
    id: usize,
    tx: Arc<ClientTx<WsResponse>>,
}

impl Connection {
    pub fn new(tx: ClientTx<WsResponse>) -> Self {
        let id = CONNECTION_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id,
            tx: Arc::new(tx),
        }
    }

    #[inline]
    pub fn send(&self, response: &WsResponse) {
        self.tx.send(response)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "content")]
enum WsRequest {
    Time,
}

#[derive(Debug)]
struct ClientRx<T>(
    futures::stream::SplitStream<WebSocket>,
    std::marker::PhantomData<T>,
);

impl<T> ClientRx<T>
where
    for<'a> T: Deserialize<'a>,
{
    pub async fn next(&mut self) -> Option<T> {
        while let Some(Ok(message)) = self.0.next().await {
            match message
                .to_str()
                .and_then(|text| serde_json::from_str(text).map_err(|_| ()))
            {
                Ok(data) => return Some(data),
                Err(_) => continue,
            }
        }

        None
    }
}

impl<T> From<futures::stream::SplitStream<WebSocket>> for ClientRx<T> {
    fn from(ws_rx: futures::stream::SplitStream<WebSocket>) -> Self {
        Self(ws_rx, Default::default())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "content")]
enum WsResponse {
    OnCreated { phrase: String },
}

#[derive(Debug, Clone)]
struct ClientTx<T>(
    mpsc::UnboundedSender<Result<ws::Message, warp::Error>>,
    std::marker::PhantomData<T>,
);

impl<T> ClientTx<T>
where
    T: Serialize,
{
    pub fn send(&self, message: &T) {
        self.send_raw(ws::Message::text(serde_json::to_string(message).unwrap()));
    }
}

impl<T> ClientTx<T> {
    #[inline]
    pub fn send_raw(&self, message: ws::Message) {
        let _ = self.0.send(Ok(message));
    }
}

impl<T> From<mpsc::UnboundedSender<Result<ws::Message, warp::Error>>> for ClientTx<T> {
    fn from(tx: mpsc::UnboundedSender<Result<ws::Message, warp::Error>>) -> Self {
        Self(tx, Default::default())
    }
}

static CONNECTION_ID: AtomicUsize = AtomicUsize::new(0);
