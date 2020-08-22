mod controllers;

use warp::Filter;

use crate::prelude::*;

pub async fn serve(settings: Arc<Settings>) {
    let ctx = Context {
        settings: settings.clone(),
    };

    let api = filters::api_v1(ctx);
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "OPTIONS", "POST", "DELETE", "PUT"]);
    let log = warp::log("fbox");

    warp::serve(api.with(log).with(cors))
        .run(settings.server_addr)
        .await
}

#[derive(Debug, Clone)]
pub struct Context {
    pub settings: Arc<Settings>,
}

mod filters {
    use super::*;

    use warp::filters::BoxedFilter;
    use warp::Filter;

    pub fn api_v1(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
        warp::path!("v1").and(post_sessions(ctx)).boxed()
    }

    fn post_sessions(ctx: Context) -> BoxedFilter<(impl warp::Reply,)> {
        warp::path!("sessions")
            .and(warp::post())
            .and(with_ctx(ctx))
            .and_then(controllers::post_sessions)
            .boxed()
    }

    fn with_ctx(
        ctx: Context,
    ) -> impl Filter<Extract = (Context,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || ctx.clone())
    }
}
