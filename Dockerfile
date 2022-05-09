# 1. This tells docker to use the Rust official image
FROM rust:1.58.1 as build

# create a new empty shell project
RUN USER=root cargo new --bin lake-sync
WORKDIR /lake-sync

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/lake_sync*
RUN cargo build --release

# our final base
FROM debian:buster-slim

# Copy from the previous build
COPY --from=build /lake-sync/target/release/lake-sync /usr/src/lake-sync
# COPY --from=build /lake-sync/target/release/lake-sync/target/x86_64-unknown-linux-musl/release/lake-sync .

# Run the binary
CMD ["/usr/src/lake-sync"]