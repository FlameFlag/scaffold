# Software Packaging Example

This example shows how Scaffold can describe a small mixed packaging setup:

- build a minimal Git from source for Linux, macOS, and Windows
- download MSYS2 and its MinGW-w64 compiler packages for Windows Git builds
- install VS Code from native package/application installers
- create Arch, Fedora, and Debian Distrobox environments
- install native packages inside each Distrobox with `pacman`, `dnf`, or `apt`
- install shared CLI tools inside each Distrobox with `nix profile install`
- add Bun and uv managed tools.

The example is intentionally declarative. Running validation does not install
Git, VS Code, or containers. It only proves the catalog evaluates, validates,
and produces the expected argv shapes.

## Validate

From the repository root:

```sh
cargo run -- --catalog examples/catalog.scm list
cargo run -- --catalog examples/catalog.scm test
cargo run -- fmt --check examples/catalog.scm examples/test.scm examples/extensions/software-packaging/catalog.scm examples/extensions/software-packaging/*/*.scm
```

You can also run the Docker validation harness from the repository root:

```sh
docker build -t scaffold-packaging-example .
docker compose build packaging-example
```

## Preparing Git Sources

The Git build tools point at `examples/sources/git`. Populate
that directory before actually running the source build actions:

```sh
examples/scripts/fetch-git-source.sh
```

Pass a version to fetch a specific upstream release:

```sh
examples/scripts/fetch-git-source.sh 2.45.2
```

The Windows source build depends on the `mingw-w64-toolchain` example tool. That
tool installs MSYS2 with WinGet, then uses MSYS2 `pacman` to download the
MinGW-w64 compiler and Git build dependencies before running the Git build
through `C:/msys64/usr/bin/bash.exe`.
