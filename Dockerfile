FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN apt-get update && apt install -y openssl
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim as runner
# Add openssl
RUN apt-get update && apt install -y openssl

FROM runner as service
COPY --from=builder /usr/local/cargo/bin/log-receiver /usr/local/bin/log-receiver
EXPOSE 8000
CMD ["log-receiver"]
