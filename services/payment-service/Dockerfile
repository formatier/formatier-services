FROM rust:alpine as builder

WORKDIR /app
COPY . .

RUN cargo build -p payment-service --release --bin payment-service

FROM gcr.io/distroless/cc-debian13

WORKDIR /bin
COPY --from=builder /app/target/release/payment-service ./

CMD ["/bin/payment-service"]
