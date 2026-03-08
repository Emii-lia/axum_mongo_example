FROM rust:1.94.0 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/axum_mongo /app/axum_mongo
EXPOSE 3000
CMD ["./axum_mongo"]