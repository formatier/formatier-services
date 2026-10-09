FROM rust:alpine as builder

WORKDIR /app
COPY . .

RUN cargo build -p monitor-server --release --bin monitor-server

FROM gcr.io/distroless/cc-debian13

WORKDIR /bin
COPY --from=builder /app/target/release/monitor-server ./

CMD ["/bin/monitor-server"]
