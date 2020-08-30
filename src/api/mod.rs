mod filters;

use warp::Filter;

use crate::prelude::*;
use crate::services::sessions::SessionService;

pub async fn serve(ctx: Context) {
    let server_addr = ctx.settings.server_addr;
    let api = filters::api_v1(ctx);
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Content-Type", "Content-Length", "X-Session-Seed"])
        .allow_methods(vec!["GET", "OPTIONS", "POST", "DELETE", "PUT"]);
    let log = warp::log("fbox");

    warp::serve(api.with(log).with(cors)).run(server_addr).await
}

#[derive(Clone)]
pub struct Context {
    pub settings: Arc<Settings>,
    pub session_service: Arc<SessionService>,
}
