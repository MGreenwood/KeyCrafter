FROM rust:latest

WORKDIR /usr/src/keycrafter
COPY . .
RUN cargo build --release

CMD cp target/release/keycrafter /output/ 