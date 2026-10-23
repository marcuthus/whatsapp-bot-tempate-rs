use std::time::Duration;

use tracing::info;

pub async fn run(code: String, timeout: Duration) {
    info!(
        target: "client",
        "Pairing code (valid for {}s): {code}",
        timeout.as_secs()
    );
}
