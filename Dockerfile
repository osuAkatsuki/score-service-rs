FROM rust:bookworm AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef --locked
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
    apt-get install -y openssl ca-certificates python3-pip git && \
    apt-get autoremove -y && \
    apt-get clean
RUN pip install --break-system-packages git+https://github.com/osuAkatsuki/akatsuki-cli

WORKDIR /app
COPY --from=builder /app/target/release/score-service /app
COPY scripts /app/scripts
ENTRYPOINT ["/app/scripts/entrypoint.sh"]
