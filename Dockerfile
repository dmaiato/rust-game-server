FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY . .

RUN rustc src/main.rs -o server && strip server

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/server .

EXPOSE 10000/udp

CMD ["./server"]