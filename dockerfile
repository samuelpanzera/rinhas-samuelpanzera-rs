#dev
FROM rust:1.88-alpine AS dev

RUN apk add --no-cache musl-dev g++ libc-dev bash curl \
  gdb lldb vim netcat openssl-dev

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /rinha
COPY . .

RUN cargo build --target x86_64-unknown-linux-musl

CMD ["sh"]

# Build
FROM rust:1.88-alpine AS builder
RUN apk add --no-cache build-base musl-dev

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /rinha
COPY . .

ENV RUSTFLAGS="-C target-feature=+crt-static"

RUN cargo build --release --target x86_64-unknown-linux-musl

#production
FROM scratch AS final

COPY --from=builder /rinha/target/x86_64-unknown-linux-musl/release/rinha-samuelpanzera-rs /rinha

ENTRYPOINT ["/rinha"]
