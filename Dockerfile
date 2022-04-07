FROM rust:1.59.0-slim as builder

WORKDIR /usr/src/
RUN apt-get update && \
    apt-get -y --no-install-recommends install pkg-config libssl-dev

RUN USER=root cargo new --bin k8s-secret-check
WORKDIR /usr/src/k8s-secret-check
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
# Cache build of deps
RUN cargo install --path .

RUN rm -Rf src && \
    rm -f target/release/deps/k8s_secret_check-*
COPY ./src/main.rs ./src/main.rs
RUN cargo install --path .


FROM debian:bullseye-slim

RUN useradd -u 1001 -s /bin/bash k8s-secret-check
USER 1001

COPY --from=builder /usr/local/cargo/bin/k8s-secret-check /usr/local/bin/k8s-secret-check
CMD ["/usr/local/bin/k8s-secret-check"]

LABEL org.opencontainers.image.source="https://github.com/cakemanny/gsm-controller"
