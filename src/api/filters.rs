use super::Context;

use crate::api::res::*;
use crate::prelude::*;

use bytes::{Buf, BufMut};
use futures::{Stream, TryFutureExt, TryStreamExt};
use warp::filters::BoxedFilter;
use warp::Filter;

pub fn api_v1(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path("v1")
        .and(
            post_sessions(ctx.clone())
                .or(post_sessions_files(ctx.clone()))
                .or(ws_sessions_socket(ctx)),
        )
        .boxed()
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

fn post_sessions_files(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("sessions" / "files")
        .and(warp::post())
        .and(warp::header::value("content-length"))
        .and(warp::filters::body::stream())
        .and_then(handle_stream)
        .boxed()
}

async fn handle_stream<T, I>(size: http::HeaderValue, form: T) -> Result<impl warp::Reply, warp::Rejection>
where
    T: Stream<Item = Result<I, warp::Error>>,
    I: Buf,
{
    use futures::stream::StreamExt;

    println!("Hello world! {:?}", size);

    form.for_each(|item| async move {
        match item {
            Ok(buf) => {
                println!("item: {:?}", buf.remaining());
                tokio::time::delay_for(tokio::time::Duration::from_secs(10)).await;
            }
            Err(e) => {
                println!("error: {}", e.to_string());
            }
        }
    })
    .await;

    Ok::<_, warp::Rejection>(warp::reply::json(&()))
}

//
// fn post_sessions_files(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
//     warp::path!("sessions" / "files")
//         .and(warp::post())
//         .and(warp::multipart::form().max_length(1024 * 1024 * 1024))
//         .and_then(|form: warp::multipart::FormData| async move {
//             use warp::Stream;
//
//             log::debug!("Got form request: {:?}", form);
//
//             println!("size_hint: {:?}", form.size_hint());
//
//             form.try_for_each(|part: warp::multipart::Part| async move {
//                 println!("part: {:?}", part);
//
//                 println!("sleeping......");
//                 tokio::time::delay_for(tokio::time::Duration::from_secs(60)).await;
//
//                 Ok(())
//             })
//                 .await;
//
//             Ok::<_, warp::Rejection>(warp::reply::json(&()))
//         })
//         .boxed()
// }

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
