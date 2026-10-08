FROM rust:alpine as builder

WORKDIR /app
COPY . .

RUN cargo build -p forum-service --release --bin forum-service

FROM gcr.io/distroless/cc-debian13

WORKDIR /bin
COPY --from=builder /app/target/release/forum-service ./

CMD ["/bin/forum-service"]
