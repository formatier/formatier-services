FROM rust:alpine as builder

WORKDIR /app
COPY . .

RUN cargo build -p formatine-worker --release --bin formatine-worker

FROM gcr.io/distroless/cc-debian13

WORKDIR /bin
COPY --from=builder /app/target/release/formatine-worker ./

CMD ["/bin/formatine-worker"]
