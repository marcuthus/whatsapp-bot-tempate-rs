use std::sync::Arc;

use tracing::info;
use whatsapp_rust::prelude::*;

pub async fn run(_client: Arc<Client>) {
    info!(target: "client", "Successfully started and connected to WhatsApp!");
}
