# AGENTS.md - WhatsApp Bot Template (Rust)

## Build/Lint/Test Commands

### Development

- `cargo run` - Run bot in development mode
- `cargo build` - Compile in debug mode
- `cargo build --release` - Compile optimized binary to `./target/release/`
- `RUST_LOG=debug cargo run` - Run with verbose console logs

### Linting & Formatting

- `cargo fmt` - Format code (config in `rustfmt.toml`)
- `cargo fmt --check` - Check formatting without changing files
- `cargo clippy --all-targets -- -D warnings` - Lint (warnings fail CI)

### Testing

- `cargo test` - Run all tests
- `cargo test <name>` - Run a single test by name

Unit tests live next to the code in `#[cfg(test)] mod tests`.

## Code Style Guidelines

### Imports

Order (separated by blank lines, `rustfmt` keeps each group sorted):

1. `std`
2. External crates (`whatsapp_rust`, `tracing`, ...)
3. Internal modules (`crate::...`)

Use `whatsapp_rust::prelude::*` for the common client types (`Bot`, `MessageContext`, `Client`, `SendResult`, `wa`, ...).

### Naming Conventions

- **Files/Modules/Functions/Variables**: `snake_case` (e.g., `uptime.rs`, `require_bot_owner`)
- **Types/Enums/Traits**: `PascalCase` (e.g., `Config`, `Data`, `Command`)
- **Constants**: `UPPER_SNAKE_CASE`
- Command names are lowercase letters, digits or underscores

### Comments

- **Never write comments in code**. Code must be 100% self-explanatory through clean structure and descriptive names.

### Error Handling

- Command and event functions return `Result<(), crate::data::Error>`; use `?` to propagate.
- Startup code returns `anyhow::Result`; config errors use `thiserror` (`ConfigError`).
- Avoid `unwrap()`/`expect()` outside tests. Never `panic!` in handlers.
- The global handler in `src/error.rs` logs the error and sends a friendly message to the user; `CommandError::PermissionDenied` produces the permission message.

## WhatsApp Bot Patterns

**Commands:**

- Declare each command as a variant of the `Command` enum in `src/commands/mod.rs`, with an entry in `CATALOG` (category, name, description; used by `!help` and `!menu`, and the category must exist in `CATEGORY_LABELS`). Arguments are the variant fields, read in order in `Command::from_parts` through helpers such as `number`; parse errors (`ParseError`) are reported to the user automatically.
- Commands are triggered by `Config::prefix` (default `!`) and are case-insensitive. Unknown commands are ignored silently, and the bot's own messages are skipped.
- Place the implementation in `src/commands/<category>/<command_name>.rs` exposing `pub async fn run(ctx: &Context, ...)`, and dispatch it in the `match` of `commands::run`.
- Restrict to the bot owner with `require_bot_owner(ctx)?` and to group admins (or the owner) with `require_group_admin(ctx).await?` (`src/utils/checks.rs`, returns the group metadata and fails with `CommandError::NotInGroup` / `PermissionDenied`); it compares `WHATSAPP_ADMIN_NUMBER` with the sender's number and LID alternate.
- Command names may contain hyphens (`add-member`); free-text arguments (e.g. `create-group <nome>`) are joined in `Command::from_parts`.
- `sticker [crop] [center]` converts images with the `image` and `webp` crates (static stickers only): `crop` is `true` (fill and crop, default), `false` (fit with transparent bars) or `dented` (stretched); `center` is `true` (default) or `false` (anchor top-left). Options are parsed by `StickerOptions::parse`.
- Replies use WhatsApp formatting (`*bold*`, `_italic_`, `~strike~`, `` `mono` ``); there is no HTML and no escaping.

**Events:**

- Place events in `src/events/<event_name>.rs` exposing `pub async fn run(...)`.
- Register them in the `Bot::builder()` chain in `src/client.rs` (`on_qr_code`, `on_pair_code`, `on_connected`, `on_logged_out`, `on_event`, ...). Incoming messages go through `on_message` into `commands::handle_message`.

**Client:**

- `client::start(config)` opens the SQLite session store (`Config::database_path`, default `auth/whatsapp.db`), builds the `whatsapp_rust` bot, optionally requests a pairing code (`WHATSAPP_PAIRING_NUMBER`), and runs it until Ctrl+C / SIGTERM, then shuts down gracefully.
- Shared state goes in `data::Data` (available as `ctx.data`); use `Arc`/atomics/`Mutex` for mutable state.
- `data::Context` bundles the incoming `MessageContext` and the `Data`, and provides `text`, `author_name`, `author_ids`, `reply` (quoting), `edit` and `react` helpers.
- `whatsapp-rust` is built with `default-features = false` because its default `simd` feature requires nightly Rust. Keep it that way to stay on stable.

## Configuration

- Environment variables loaded via `dotenvy` and validated by `Config::from_env()` (`src/helpers/config.rs`).
- Required env vars: `WHATSAPP_ADMIN_NUMBER` (digits with country code, e.g. `5511999999999`). Optional: `WHATSAPP_PAIRING_NUMBER`.

## Logging

- Use `tracing` macros with a scope as target: `info!(target: "client", "...")`, `error!`, `warn!`, `debug!`.
- Logs go to console and to `logs/combined.log` / `logs/error.log` (see `src/helpers/logger.rs`). `whatsapp_rust` and `wacore` are limited to `warn` by default.
- Keep the `WorkerGuard`s returned by `logger::init()` alive in `main`.

## Docker

- `Dockerfile` is multi-stage (`cargo build --release` with a cached dependency layer, runtime on `debian:bookworm-slim`) and runs as an unprivileged user.
- `docker-compose.yml` loads `.env` through `env_file` and keeps `logs/` and `auth/` (the WhatsApp session) in named volumes.
- `.dockerignore` keeps `.env`, `logs/`, `auth/`, `.git` and build output out of the image. Never copy secrets into the image.
- Build and run: `docker compose up --build`.
