# Scaffold Examples

This folder is a compact tour of a real Scaffold catalog. The examples are
split into small modules so each file shows one idea clearly instead of hiding
everything in one large scenario.

## Layout

- [`catalog.scm`](catalog.scm) is the root catalog entry point.
- [`test.scm`](test.scm) contains pure assertions for the example helpers.
- [`extensions/examples/catalog.scm`](extensions/examples/catalog.scm) composes
  the focused tool groups into the root catalog.
- [`extensions/examples/tools/`](extensions/examples/tools/) contains focused
  tool groups.
- [`extensions/examples/targets/distrobox.scm`](extensions/examples/targets/distrobox.scm)
  shows how to derive wrapped tools with catalog transformations.
- [`sources/hello/`](sources/hello/) is a tiny source tree used by the workspace
  build example.

## Validate

From the repository root:

```sh
cargo run -- --catalog examples/catalog.scm list
cargo run -- --catalog examples/catalog.scm test
cargo run -- fmt --check examples/catalog.scm examples/test.scm examples/extensions/**/*.scm
```

The tests do not install anything. They validate that the catalog evaluates and
that key generated argv shapes stay stable.

## Try Individual Tools

List the example catalog:

```sh
cargo run -- --catalog examples/catalog.scm list
```

Install a specific tool when you are on a machine where the install action makes
sense:

```sh
cargo run -- --catalog examples/catalog.scm install ripgrep
cargo run -- --catalog examples/catalog.scm install prettier
```

Inspect paths and discovered local extension modules:

```sh
cargo run -- --catalog examples/catalog.scm paths --sources
```
