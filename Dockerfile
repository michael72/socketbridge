FROM rust:1.83.0-slim AS socketbridge-build

RUN mkdir -pv /opt/socketbridge/src
WORKDIR /opt/socketbridge
COPY Cargo.toml .
COPY src/main.rs src/main.rs
RUN cargo build --release

FROM rust:1.83.0-slim AS socketbridge

RUN mkdir -pv /opt/socketbridge
WORKDIR /opt/socketbridge
COPY --from=socketbridge-build /opt/socketbridge/target/release/socketbridge .
