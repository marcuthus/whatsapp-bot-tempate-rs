mod client;
mod commands;
mod data;
mod error;
mod events;
mod helpers;
mod utils;

use helpers::{config::Config, logger};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let _log_guards = logger::init();
    let config = Config::from_env()?;

    client::start(config).await
}
