FROM rust:1.83.0-slim AS square-server-build

RUN mkdir -pv /opt/square_server
WORKDIR /opt/square_server
COPY Cargo.toml .
COPY src src
RUN cargo build --release

FROM rust:1.83.0-slim AS square-server

RUN mkdir -pv /opt/square_server
WORKDIR /opt/square_server
COPY --from=docker.io/library/socketbridge /opt/socketbridge/socketbridge .
COPY --from=square-server-build /opt/square_server/target/release/square_server .
COPY ./scripts/start_server.sh .

ENTRYPOINT [ "./start_server.sh" ]
