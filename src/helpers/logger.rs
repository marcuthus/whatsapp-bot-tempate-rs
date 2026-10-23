use tracing_appender::{non_blocking, non_blocking::WorkerGuard};
use tracing_subscriber::{
    EnvFilter, Layer, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

pub fn init() -> Vec<WorkerGuard> {
    let console_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,whatsapp_rust=warn,wacore=warn"));

    let (combined, combined_guard) =
        non_blocking(tracing_appender::rolling::never("logs", "combined.log"));
    let (errors, errors_guard) =
        non_blocking(tracing_appender::rolling::never("logs", "error.log"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_filter(console_filter))
        .with(
            fmt::layer()
                .with_ansi(false)
                .with_writer(combined)
                .with_filter(LevelFilter::INFO),
        )
        .with(
            fmt::layer()
                .with_ansi(false)
                .with_writer(errors)
                .with_filter(LevelFilter::ERROR),
        )
        .init();

    vec![combined_guard, errors_guard]
}
