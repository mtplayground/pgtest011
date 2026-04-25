FROM rust:1.95-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        binaryen \
        ca-certificates \
        curl \
        libssl-dev \
        npm \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos --locked
RUN npm install -g sass

WORKDIR /app

COPY . .

RUN cargo leptos build --release


FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 appuser

WORKDIR /app

ENV APP_ENV=production
ENV HOST=0.0.0.0
ENV PORT=8080
ENV LEPTOS_OUTPUT_NAME=pgtest011
ENV LEPTOS_SITE_ROOT=site
ENV LEPTOS_SITE_PKG_DIR=pkg
ENV LEPTOS_SITE_ADDR=0.0.0.0:8080
ENV LEPTOS_RELOAD_PORT=3001

COPY --from=builder /app/Cargo.toml /app/Cargo.toml
COPY --from=builder /app/target/server/release/pgtest011 /app/pgtest011
COPY --from=builder /app/target/site /app/site

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD curl --fail http://127.0.0.1:8080/healthz || exit 1

CMD ["./pgtest011"]
