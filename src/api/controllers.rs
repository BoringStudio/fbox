use super::Context;

use futures::future::FutureExt;

use crate::api::resp::*;
use crate::prelude::*;

pub fn post_sessions(
    ctx: Context,
) -> BoxFuture<'static, Result<impl warp::Reply, warp::Rejection>> {
    async move {
        let mnemonic_resp = MnemonicResp::from(ctx.session_service.generate_mnemonic());

        Ok(warp::reply::json(&mnemonic_resp))
    }
    .boxed()
}
