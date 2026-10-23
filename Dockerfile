FROM rust:1-bookworm AS build
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates ffmpeg libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home app

WORKDIR /app

COPY --from=build /app/target/release/whatsapp-bot-rs /usr/local/bin/whatsapp-bot-rs

RUN mkdir -p logs auth && chown -R app:app /app
USER app

CMD ["whatsapp-bot-rs"]
