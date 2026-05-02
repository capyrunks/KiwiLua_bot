FROM rust:slim AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Используем * для опционального копирования Cargo.lock (чтобы сборка не падала, если его нет)
COPY Cargo.toml Cargo.lock* ./

RUN mkdir src \
    && printf 'fn main() {}\n' > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
# Обязательно обновляем время модификации main.rs, иначе Cargo может не пересобрать бинарник из-за кэша!
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 --shell /usr/sbin/nologin appuser

WORKDIR /app

COPY --from=builder /app/target/release/kiwilua-bot /usr/local/bin/kiwilua-bot

# Создаем пустую папку lua_files, чтобы избежать ошибки "not found", если ее нет в Git
RUN mkdir -p ./lua_files && chown appuser:appuser ./lua_files
# Копируем весь контекст во временную директорию и переносим lua_files, если она существует
COPY --chown=appuser:appuser . /tmp_context/
RUN if [ -d /tmp_context/lua_files ]; then cp -r /tmp_context/lua_files/* ./lua_files/ 2>/dev/null || true; fi && rm -rf /tmp_context

RUN mkdir -p /app/data \
    && chown -R appuser:appuser /app /usr/local/bin/kiwilua-bot

USER appuser

ENV PORT=7860
ENV RUST_LOG=info

EXPOSE 7860

CMD ["kiwilua-bot"]
