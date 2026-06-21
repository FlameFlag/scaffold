# syntax=docker/dockerfile:1

ARG BUN_VERSION=1.3.14
ARG WASM_BINDGEN_VERSION=0.2.123

FROM oven/bun:${BUN_VERSION} AS bun

FROM rust:1-bookworm AS verify

ARG WASM_BINDGEN_VERSION

WORKDIR /workspace

COPY --from=bun /usr/local/bin/bun /usr/local/bin/bun
COPY . .

ENV CARGO_TERM_COLOR=always

RUN rustup target add wasm32-unknown-unknown && rustup component add clippy rustfmt
RUN cargo install wasm-bindgen-cli --locked --version "$WASM_BINDGEN_VERSION"
RUN bun install --frozen-lockfile
RUN cargo fmt --all -- --check
RUN cargo clippy --workspace --all-targets --locked -- -D warnings
RUN cargo build --workspace --locked
RUN cargo test --workspace --locked
RUN bun run check
RUN bun run fallow:ci
RUN bun run docs:check
RUN bun run wasm:reference:check
RUN bun run vscode:check
RUN bun run vscode:compile:check
RUN bun run vscode:compile
RUN bun run vscode:vsix:pack:check
RUN bun run vscode:test:wasm
RUN cargo run --locked -- fmt --check examples/catalog.scm examples/test.scm examples/extensions/examples/catalog.scm examples/extensions/examples/*/*.scm
RUN cargo run --locked -- --catalog examples/catalog.scm list
RUN cargo run --locked -- --catalog examples/catalog.scm test
