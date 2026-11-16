# Ships with cargo-chef preinstalled, which saves compiling it on every uncached build
FROM lukemathwalker/cargo-chef:0.1.78-rust-1.100-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin score-service

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
# Install openssl, root certificates, pip and git
RUN apt-get update && \
    apt-get install -y --no-install-recommends openssl ca-certificates python3-pip git && \
    apt-get autoremove -y && \
    apt-get clean
# Pinned so that cached layers and fresh builds install the same version,
# bump the commit to pick up changes
RUN pip install --break-system-packages git+https://github.com/osuAkatsuki/akatsuki-cli@5420cb6ebf58a4ac6e06feeac853c3ebe68c7836

WORKDIR /app
COPY --from=builder /app/target/release/score-service /app
COPY scripts /app/scripts
ENTRYPOINT ["/app/scripts/entrypoint.sh"]
