use std::{path::Path, sync::Arc, time::Instant};

use tracing::info;
use whatsapp_rust::{pair_code::PairCodeOptions, prelude::*};

use crate::{commands, data::Data, events, helpers::config::Config};

pub async fn start(config: Config) -> anyhow::Result<()> {
    info!(target: "client", "Starting...");

    if let Some(directory) = Path::new(&config.database_path).parent() {
        std::fs::create_dir_all(directory)?;
    }

    let store = SqliteStore::new(&config.database_path).await?;
    let pairing_number = config.pairing_number.clone();

    let data = Arc::new(Data {
        config: Arc::new(config),
        started_at: Instant::now(),
    });

    let mut builder = Bot::builder()
        .with_backend(store)
        .skip_history_sync()
        .on_qr_code(events::qr_code::run)
        .on_pair_code(events::pair_code::run)
        .on_connected(events::ready::run)
        .on_logged_out(events::logged_out::run)
        .on_message(move |message| commands::handle_message(message, data.clone()));

    if let Some(phone_number) = pairing_number {
        builder = builder.with_pair_code(PairCodeOptions {
            phone_number,
            ..Default::default()
        });
    }

    let handle = builder.build().await?.spawn();

    shutdown_signal().await;

    info!(target: "client", "Stopping...");
    handle.shutdown().await;
    info!(target: "client", "Successfully stopped!");

    Ok(())
}
