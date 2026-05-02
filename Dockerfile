FROM rust:slim AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
RUN mkdir src \
    && printf 'fn main() {}\n' > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 --shell /usr/sbin/nologin appuser

WORKDIR /app

COPY --from=builder /app/target/release/kiwilua-bot /usr/local/bin/kiwilua-bot

RUN mkdir -p /app/data \
    && chown -R appuser:appuser /app /usr/local/bin/kiwilua-bot

USER appuser

ENV PORT=7860
ENV RUST_LOG=info
ENV LUA_SOURCE_MAX_BYTES=52428800
ENV LUA_SOURCE_TIMEOUT_SECS=20
ENV LUA_SOURCE_URL_TEMPLATES="https://api.printedwaste.com/gfk/download/{app_id};https://cysaw.top/uploads/{app_id}.zip;https://furcate.eu/files/{app_id}.zip;https://raw.githubusercontent.com/Steam-Tools/Ryuu/main/database/{app_id}.lua"

EXPOSE 7860

CMD ["kiwilua-bot"]
