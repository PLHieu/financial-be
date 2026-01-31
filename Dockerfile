# Stage 1: build
FROM rust:1-bookworm AS builder

WORKDIR /app

# Copy manifests and source
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Stage 2: runtime
FROM debian:bookworm-slim

RUN apt-get update -y && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/financial-be /usr/local/bin/financial-be

RUN useradd -m -u 1000 app && chown app:app /usr/local/bin/financial-be
USER app

EXPOSE 3001

CMD ["/usr/local/bin/financial-be"]
