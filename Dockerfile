# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS verify

WORKDIR /workspace

COPY . .

ENV CARGO_TERM_COLOR=always

RUN cargo build --locked
RUN cargo test --locked
RUN cargo run --locked -- fmt --check examples/catalog.scm examples/test.scm examples/extensions/software-packaging/catalog.scm examples/extensions/software-packaging/*/*.scm
RUN cargo run --locked -- --catalog examples/catalog.scm list
RUN cargo run --locked -- --catalog examples/catalog.scm test
