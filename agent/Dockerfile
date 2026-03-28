FROM rust:1.75

WORKDIR /app
COPY . .

RUN cargo build --release

CMD ["./target/release/poc"]
