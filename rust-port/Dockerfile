FROM registry.access.redhat.com/ubi9/ubi:latest AS builder
WORKDIR /app

RUN dnf install -y \
        ca-certificates \
        gcc \
        gcc-c++ \
        git \
        make \
        openssl-devel \
        perl-core \
        pkgconf-pkg-config \
        tar \
    && dnf clean all

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH}

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates

RUN cargo build --locked --release -p backend

FROM registry.access.redhat.com/ubi9/ubi-minimal:latest AS runtime
RUN microdnf install -y ca-certificates tzdata shadow-utils \
    && microdnf clean all \
    && update-ca-trust \
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
