FROM rust:latest

WORKDIR /app

COPY . .
COPY Cargo.toml Cargo.lock ./
COPY backend/src ./src

RUN cargo build --release

CMD ["./target/release/rwayve"]