use std::time::Duration;

use qrcode::{QrCode, render::unicode};
use tracing::{error, info};

pub async fn run(code: String, timeout: Duration) {
    match QrCode::new(code.as_bytes()) {
        Ok(qr) => {
            let rendered = qr.render::<unicode::Dense1x2>().build();

            info!(
                target: "client",
                "Scan the QR code below with WhatsApp (Linked devices), valid for {}s:\n{rendered}",
                timeout.as_secs()
            );
        }
        Err(render_error) => {
            error!(target: "client", "Failed to render the QR code: {render_error}");
        }
    }
}
