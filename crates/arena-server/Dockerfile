FROM rust:1-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --locked --release -p arena-server

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 arena \
    && useradd --uid 10001 --gid arena --no-create-home arena \
    && mkdir /data \
    && chown arena:arena /data
COPY --from=build /app/target/release/arena-server /usr/local/bin/arena-server
USER arena
WORKDIR /data
ENV BIND_ADDR=0.0.0.0:3000 \
    DATABASE_URL=sqlite:///data/arena.db
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s \
    CMD curl --fail --silent http://127.0.0.1:3000/api/health || exit 1
CMD ["arena-server"]
