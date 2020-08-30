use std::sync::atomic::{AtomicUsize, Ordering};

use futures::future::Ready;
use futures::task::{Context, Poll};
use futures::{FutureExt, Stream, StreamExt};
use warp::ws::{self, WebSocket};

use crate::prelude::*;

pub fn init_connection<Int, ExtReq, ExtRes>(
    websocket: WebSocket,
) -> (Arc<Connection<Int, ExtRes>>, EventRx<Int, ExtReq, impl Stream<Item = ExtReq>>)
where
    Int: Send,
    for<'a> ExtReq: Deserialize<'a> + Send,
    ExtRes: Serialize + Send,
{
    let (ws_tx, external_rx) = websocket.split();
    let (external_tx, rx) = mpsc::unbounded_channel();
    let (internal_tx, internal_rx) = mpsc::unbounded_channel();

    tokio::task::spawn(rx.forward(ws_tx).map(|result| {
        if let Err(e) = result {
            log::trace!("websocket closed: {}", e);
        }
    }));

    let connection = Arc::new(Connection::new(internal_tx, external_tx));
    let event_rx = EventRx::new(internal_rx, external_rx.filter_map(filter_external));

    (connection, event_rx)
}

#[derive(Debug, Clone)]
pub struct Connection<Int, ExtRes> {
    id: ConnectionId,
    internal_tx: InternalTx<Int>,
    external_tx: WebSocketTx,
    _marker: std::marker::PhantomData<ExtRes>,
}

impl<Int, ExtRes> Connection<Int, ExtRes>
where
    Int: Send,
    ExtRes: Serialize,
{
    fn new(internal_tx: InternalTx<Int>, external_tx: WebSocketTx) -> Self {
        let id = CONNECTION_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id,
            internal_tx,
            external_tx,
            _marker: Default::default(),
        }
    }

    #[inline]
    pub fn send_internal(&self, message: Int) {
        let _ = self.internal_tx.send(message);
    }

    #[inline]
    pub fn send_external(&self, message: &ExtRes) {
        self.send_external_raw(ws::Message::text(serde_json::to_string(message).unwrap()));
    }

    #[inline]
    pub fn send_external_raw(&self, message: ws::Message) {
        let _ = self.external_tx.send(Ok(message));
    }

    #[inline]
    pub fn id(&self) -> ConnectionId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub enum Event<Int, ExtReq> {
    Internal(Int),
    External(ExtReq),
}

#[pin_project]
#[derive(Debug)]
pub struct EventRx<Int, ExtReq, ExtReqStr>
where
    ExtReqStr: Stream<Item = ExtReq>,
{
    #[pin]
    internal_rx: InternalRx<Int>,
    #[pin]
    external_rx: ExtReqStr,
    _marker: std::marker::PhantomData<ExtReq>,
}

impl<Int, ExtReq, ExtReqStr> EventRx<Int, ExtReq, ExtReqStr>
where
    Int: Send,
    for<'a> ExtReq: Deserialize<'a>,
    ExtReqStr: Stream<Item = ExtReq>,
{
    fn new(internal_rx: InternalRx<Int>, external_rx: ExtReqStr) -> Self {
        Self {
            internal_rx,
            external_rx,
            _marker: Default::default(),
        }
    }
}

fn filter_external<ExtReq>(item: WebSocketRxItem) -> Ready<Option<ExtReq>>
where
    for<'a> ExtReq: Deserialize<'a> + Send,
{
    futures::future::ready(
        item.ok()
            .and_then(|message| message.to_str().ok().and_then(|text| serde_json::from_str(text).ok())),
    )
}

impl<Int, ExtReq, ExtReqStr> futures::Stream for EventRx<Int, ExtReq, ExtReqStr>
where
    Int: Send,
    for<'a> ExtReq: Deserialize<'a> + Send,
    ExtReqStr: Stream<Item = ExtReq>,
{
    type Item = Event<Int, ExtReq>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        poll_event(this.internal_rx, this.external_rx, cx)
    }
}

fn poll_event<Int, ExtReq, ExtReqStr>(
    internal_rx: Pin<&mut InternalRx<Int>>,
    external_rx: Pin<&mut ExtReqStr>,
    cx: &mut Context<'_>,
) -> Poll<Option<Event<Int, ExtReq>>>
where
    Int: Send,
    for<'a> ExtReq: Deserialize<'a> + Send,
    ExtReqStr: Stream<Item = ExtReq>,
{
    if let Poll::Ready(Some(item)) = internal_rx.poll_next(cx) {
        return Poll::Ready(Some(Event::Internal(item)));
    };

    match external_rx.poll_next(cx) {
        Poll::Ready(Some(item)) => Poll::Ready(Some(Event::External(item))),
        Poll::Ready(None) => Poll::Ready(None),
        Poll::Pending => Poll::Pending,
    }
}

type WebSocketTx = mpsc::UnboundedSender<Result<ws::Message, warp::Error>>;
type WebSocketRxItem = Result<ws::Message, warp::Error>;
type InternalTx<T> = mpsc::UnboundedSender<T>;
type InternalRx<T> = mpsc::UnboundedReceiver<T>;

pub type ConnectionId = usize;

static CONNECTION_ID: AtomicUsize = AtomicUsize::new(0);
