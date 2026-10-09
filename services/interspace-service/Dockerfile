FROM rust:alpine as builder

WORKDIR /app
COPY . .

RUN cargo build -p interspace-service --release --bin interspace-service

FROM gcr.io/distroless/cc-debian13

WORKDIR /bin
COPY --from=builder /app/target/release/interspace-service ./

CMD ["/bin/interspace-service"]
