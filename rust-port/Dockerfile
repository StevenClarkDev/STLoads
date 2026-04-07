FROM rust:stable-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates

RUN cargo build --locked --release -p backend

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 appuser

WORKDIR /app
COPY --from=builder /app/target/release/backend /usr/local/bin/backend

ENV APP_ENV=production \
    DEPLOYMENT_TARGET=ibm-code-engine \
    BIND_ADDR=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=backend=info,tower_http=info

EXPOSE 8080
USER 10001
CMD ["/usr/local/bin/backend"]
