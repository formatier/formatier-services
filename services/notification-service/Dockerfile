FROM rust:alpine as builder

WORKDIR /app
COPY . .

RUN cargo build -p notification-service --release --bin notification-service

FROM gcr.io/distroless/cc-debian13

WORKDIR /bin
COPY --from=builder /app/target/release/notification-service ./

CMD ["/bin/notification-service"]
