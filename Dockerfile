FROM rust:1-bullseye AS chef
RUN cargo install cargo-chef 
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

RUN apt-get update && apt-get install -y build-essential cmake clang \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --bin aero2solver

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y libgomp1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY ./model/ ./model/
COPY --from=builder /app/target/release/aero2solver ./aero2solver

ENTRYPOINT [ "/app/aero2solver" ]
