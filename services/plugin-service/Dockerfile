FROM rust:alpine as builder

WORKDIR /app
COPY . .

RUN cargo build -p plugin-service --release --bin plugin-service

FROM gcr.io/distroless/cc-debian13

WORKDIR /bin
COPY --from=builder /app/target/release/plugin-service ./

CMD ["/bin/plugin-service"]
