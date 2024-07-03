FROM rust:1.76.0 AS base
RUN cargo install cargo-chef --locked
RUN apt-get update && apt-get install -y \
    pkg-config \
    openssl \
    libssl-dev

FROM base AS plan

WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base as build
WORKDIR /app
COPY --from=plan /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

RUN cargo build --release  --locked

FROM debian:latest as runtime

RUN apt-get update && apt-get install -y \
    openssl \
    libssl-dev \
    curl \
    make \
    && rm -rf /var/lib/apt/lists/

WORKDIR /app

COPY --from=build /app/target/release/address-monitor /usr/local/bin

CMD ["/usr/local/bin/address-monitor", "", ""]
