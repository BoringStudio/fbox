use super::Context;

use futures::future::FutureExt;

use crate::prelude::*;

pub fn post_sessions(
    _ctx: Context,
) -> BoxFuture<'static, Result<impl warp::Reply, warp::Rejection>> {
    async {
        let test_response = "Hello world".to_owned();

        Ok(warp::reply::json(&test_response))
    }
    .boxed()
}
