use tracing::error;
use whatsapp_rust::types::events::LoggedOut;

pub async fn run(event: LoggedOut) {
    error!(
        target: "client",
        "Logged out ({:?}). Delete the `auth` folder and start again.",
        event.reason
    );
}
