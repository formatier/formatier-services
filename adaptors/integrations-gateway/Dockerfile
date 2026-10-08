FROM rust:alpine as builder

WORKDIR /app
COPY . .

RUN cargo build -p integrations-gateway --release --bin integrations-gateway

FROM gcr.io/distroless/cc-debian13

WORKDIR /bin
COPY --from=builder /app/target/release/integrations-gateway ./

CMD ["/bin/integrations-gateway"]
