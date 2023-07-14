FROM rust:1.70.0 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/officermazy /usr/local/bin/officermazy
COPY .env .
CMD ["officermazy"]