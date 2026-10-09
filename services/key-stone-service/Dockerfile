FROM rust:alpine as builder

WORKDIR /app
COPY . .

RUN cargo build -p key-stone-service --release --bin key-stone-service

FROM gcr.io/distroless/cc-debian13

WORKDIR /bin
COPY --from=builder /app/target/release/key-stone-service ./

CMD ["/bin/key-stone-service"]
