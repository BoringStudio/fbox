use super::Context;

use crate::api::res::*;
use crate::prelude::*;

use warp::filters::BoxedFilter;
use warp::Filter;

pub fn api_v1(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path("v1").and(post_sessions(ctx.clone()).or(ws_sessions_socket(ctx))).boxed()
}

fn post_sessions(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("sessions")
        .and(warp::post())
        .and(with_ctx(ctx))
        .and_then(|ctx: Context| async move {
            let mnemonic_resp = MnemonicResp::from(ctx.session_service.generate_mnemonic());

            Ok::<_, warp::Rejection>(warp::reply::json(&mnemonic_resp))
        })
        .boxed()
}

fn ws_sessions_socket(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("sessions" / "socket")
        .and(warp::ws())
        .and(with_ctx(ctx))
        .map(|ws: warp::ws::Ws, ctx: Context| {
            ws.on_upgrade(move |websocket| async move { ctx.session_service.handle_connection(websocket).await })
        })
        .boxed()
}

fn with_ctx(ctx: Context) -> impl Filter<Extract = (Context,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ctx.clone())
}
