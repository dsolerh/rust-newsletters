FROM rust:1.77.0

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

ENTRYPOINT [ "./target/release/rust-newsletters" ]