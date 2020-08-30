mod api;
mod prelude;
mod services;
mod settings;

use crate::api::Context;
use crate::prelude::*;
use crate::services::sessions::SessionService;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    run().await
}

async fn run() -> Result<()> {
    let settings = Arc::new(Settings::new()?);
    let session_service = SessionService::new(&settings);

    let ctx = Context { settings, session_service };

    tokio::spawn(api::serve(ctx));

    futures::future::pending().await
}

fn init_logger() {
    let log_filters = std::env::var("RUST_LOG").unwrap_or_default();

    env_logger::Builder::new()
        .parse_filters(&log_filters)
        .format(|formatter, record| {
            use std::io::Write;

            writeln!(
                formatter,
                "{} [{}] - {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init()
}
