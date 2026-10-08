FROM rust:alpine as builder

WORKDIR /app
COPY . .

RUN cargo build -p mail-server --release --bin mail-server

FROM gcr.io/distroless/cc-debian13

WORKDIR /bin
COPY --from=builder /app/target/release/mail-server ./

CMD ["/bin/mail-server"]
