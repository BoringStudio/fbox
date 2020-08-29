use super::Context;

use crate::api::res::*;

use http::HeaderValue;
use serde::Deserialize;
use uuid::Uuid;
use warp::filters::BoxedFilter;
use warp::Filter;

pub fn api_v1(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path("v1")
        .and(
            post_sessions(ctx.clone())
                .or(get_sessions_files(ctx.clone()))
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

fn get_sessions_files(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
    #[derive(Debug, Deserialize)]
    struct Params {
        session_seed: String,
    }

    warp::path!("sessions" / "files" / Uuid)
        .and(warp::get())
        .and(warp::query::<Params>())
        .and(with_ctx(ctx))
        .and_then(|id: Uuid, params: Params, ctx: Context| async move {
            use futures::StreamExt;

            log::debug!("Received file get: {}, {:?}", id, params);

            match ctx.session_service.request_file(id, params.session_seed).await {
                Some((file_info, rx)) => {
                    let body: hyper::Body = hyper::Body::wrap_stream(rx.map(|part| Ok::<_, std::convert::Infallible>(part)));
                    hyper::Response::builder()
                        .header(
                            http::header::CONTENT_DISPOSITION,
                            format!("attachment; filename=\"{}\"", file_info.name.replace('"', "\"")),
                        )
                        .body(body)
                        .map_err(|e| {
                            println!("error: {:?}", e);
                            warp::reject()
                        })
                }
                None => Err(warp::reject()),
            }
        })
        .boxed()
}

fn post_sessions_files(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("sessions" / "files" / Uuid)
        .and(warp::post())
        .and(warp::header::value("X-Session-Seed"))
        .and(warp::header::value("Content-Length"))
        .and(warp::filters::body::stream())
        .and(with_ctx(ctx))
        .and_then(|id: Uuid, seed: HeaderValue, size: HeaderValue, data, ctx: Context| async move {
            println!("Downloading file: {:?} bytes", size);

            let seed = match seed.to_str().ok() {
                Some(seed) => seed.to_owned(),
                None => return Err(warp::reject()),
            };

            match ctx.session_service.upload_file(id, seed, data).await {
                Some(_) => Ok(warp::reply()),
                None => Err(warp::reject()),
            }
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
