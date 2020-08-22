mod api;
mod prelude;
mod settings;

use prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    run().await
}

async fn run() -> Result<()> {
    let settings = Arc::new(Settings::new()?);

    tokio::spawn(api::serve(settings.clone()));
    log::debug!("server is listening on {:?}", settings.server_addr);

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
