FROM rust:alpine as builder

WORKDIR /app
COPY . .

RUN cargo build -p api-gateway --release --bin api-gateway

FROM gcr.io/distroless/cc-debian13

WORKDIR /bin
COPY --from=builder /app/target/release/api-gateway ./

COPY --from=builder /app/deployment/config/route.yml /bin/deployment/config/route.yml

CMD ["/bin/api-gateway"]
