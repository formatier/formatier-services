FROM rust:alpine as builder

WORKDIR /app
COPY . .

RUN cargo build -p formatine-operator --release --bin formatine-operator

FROM gcr.io/distroless/cc-debian13

WORKDIR /bin
COPY --from=builder /app/target/release/formatine-operator ./

CMD ["/bin/formatine-operator"]
