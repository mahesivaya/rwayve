FROM rust:latest

WORKDIR /app

# Install tools
# RUN rustup component add clippy rustfmt

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY . .

# 🔥 SQLx compile-time validation (offline mode)
# ENV SQLX_OFFLINE=true

# 🔥 FORMAT CHECK (fail if not formatted)
# RUN cargo fmt --all -- --check


# 🔥 CLIPPY STRICT (fail build if lint fails)
# 🔥 LINT CHECK (fail if issues)
# RUN cargo clippy --all-targets --all-features -- -D warnings


RUN cargo build --release
CMD ["./target/release/rwayve"]