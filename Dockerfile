FROM rust:latest

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

# The command name must match the name of the compiled Rust
# binary.
CMD ["WebSocket-EchoServer"]