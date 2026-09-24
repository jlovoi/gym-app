FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# async-stripe "full" needs >1GB for a single rustc invocation in release mode.
# Serialize builds to minimize peak memory, and increase codegen-units to
# split each crate into smaller chunks (trades runtime perf for lower RAM).
# ENV CARGO_BUILD_JOBS=1
# ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin gym-backend

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/gym-backend /usr/local/bin
ENTRYPOINT ["/usr/local/bin/gym-backend"]
