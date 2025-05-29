FROM rust:1.87.0-slim AS sccache

RUN apt-get update && \
    apt-get -y --no-install-recommends install curl

ENV BASE_URL=https://github.com/mozilla/sccache/releases/download \
    VERSION=v0.10.0
RUN curl -LO "$BASE_URL/${VERSION}/sccache-${VERSION}-$(uname -m)-unknown-linux-musl.tar.gz.sha256"
RUN curl -LO "$BASE_URL/${VERSION}/sccache-${VERSION}-$(uname -m)-unknown-linux-musl.tar.gz"
RUN set -eu;\
    h=$(sha256sum sccache-${VERSION}-$(uname -m)-unknown-linux-musl.tar.gz | awk '{print $1}'); \
    echo "$h"; \
    g=$(cat sccache-${VERSION}-$(uname -m)-unknown-linux-musl.tar.gz.sha256); \
    echo "$g"; \
    test "$h" = "$g"
RUN tar -tvf sccache-*.tar.gz
RUN tar -xvf sccache-*.tar.gz \
    && mv sccache-*/sccache /usr/bin/sccache

FROM rust:1.87.0-slim AS builder
COPY --from=sccache /usr/bin/sccache /usr/bin/sccache
ENV RUSTC_WRAPPER=/usr/bin/sccache
ARG SCCACHE_GHA_ENABLED=off
ENV SCCACHE_GHA_ENABLED=${SCCACHE_GHA_ENABLED}
ARG ACTIONS_RESULTS_URL
ENV ACTIONS_RESULTS_URL=${ACTIONS_RESULTS_URL}

WORKDIR /usr/src/
RUN apt-get update && \
    apt-get -y --no-install-recommends install pkg-config libssl-dev

RUN USER=root cargo new --bin k8s-secret-check
WORKDIR /usr/src/k8s-secret-check
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
# Cache build of deps
RUN --mount=type=secret,id=ACTIONS_RUNTIME_TOKEN,env=ACTIONS_RUNTIME_TOKEN \
    cargo install --path . \
    && /usr/bin/sccache --show-stats

RUN rm -Rf src && \
    rm -f target/release/deps/k8s_secret_check-*
COPY ./src/main.rs ./src/main.rs
RUN --mount=type=secret,id=ACTIONS_RUNTIME_TOKEN,env=ACTIONS_RUNTIME_TOKEN \
    cargo install --path . \
    && /usr/bin/sccache --show-stats

FROM gcr.io/distroless/cc:nonroot

COPY --from=builder /usr/local/cargo/bin/k8s-secret-check /usr/local/bin/k8s-secret-check
CMD ["/usr/local/bin/k8s-secret-check"]

LABEL org.opencontainers.image.source="https://github.com/cakemanny/k8s-secret-check"
