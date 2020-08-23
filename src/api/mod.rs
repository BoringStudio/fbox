mod controllers;
mod resp;

use warp::Filter;

use crate::prelude::*;
use crate::services::sessions::SessionService;

pub async fn serve(ctx: Context) {
    let server_addr = ctx.settings.server_addr;
    let api = filters::api_v1(ctx);
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "OPTIONS", "POST", "DELETE", "PUT"]);
    let log = warp::log("fbox");

    warp::serve(api.with(log).with(cors)).run(server_addr).await
}

#[derive(Clone)]
pub struct Context {
    pub settings: Arc<Settings>,
    pub session_service: Arc<SessionService>,
}

mod filters {
    use super::*;

    use warp::filters::BoxedFilter;
    use warp::Filter;

    pub fn api_v1(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
        warp::path("v1")
            .and(post_sessions(ctx.clone()).or(ws_sessions_socket(ctx)))
            .boxed()
    }

    fn post_sessions(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
        warp::path!("sessions")
            .and(warp::post())
            .and(with_ctx(ctx))
            .and_then(controllers::post_sessions)
            .boxed()
    }

    fn ws_sessions_socket(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
        warp::path!("sessions" / "socket")
            .and(warp::ws())
            .and(with_ctx(ctx))
            .map(|ws: warp::ws::Ws, ctx: Context| {
                ws.on_upgrade(move |websocket| async move {
                    ctx.session_service.handle_connection(websocket).await
                })
            })
            .boxed()
    }

    fn with_ctx(
        ctx: Context,
    ) -> impl Filter<Extract = (Context,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || ctx.clone())
    }
}
