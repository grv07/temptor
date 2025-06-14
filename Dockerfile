FROM rust:1.87.0-alpine

RUN apk add --no-cache musl-dev libc-dev make gcc

WORKDIR /app

COPY Cargo.toml ./
COPY Cargo.lock ./
COPY entity .

RUN rm -r src

RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -r src

COPY . .

RUN cargo build --release || true

CMD ["./target/release/temptor"]
