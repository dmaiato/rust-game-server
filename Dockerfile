FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/game_server .

COPY resources/ resources/

EXPOSE 10000/udp

CMD ["./game_server"]