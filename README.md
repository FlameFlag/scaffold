<div align="center">

<h1>Scaffold</h1>

<p>
  <strong>A Scheme-driven system scaffolding tool for describing and mutating your setup the way you want.</strong>
</p>

<p>
  <a href="./LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue?style=flat-square"></a>
  <img alt="Rust 2024" src="https://img.shields.io/badge/rust-2024-f74c00?style=flat-square&logo=rust&logoColor=white">
  <img alt="Scheme DSL" src="https://img.shields.io/badge/dsl-Scheme-6e56cf?style=flat-square">
  <img alt="MCP server" src="https://img.shields.io/badge/MCP-server-00a67e?style=flat-square">
  <img alt="Status: personal project" src="https://img.shields.io/badge/status-personal%20project-59636e?style=flat-square">
</p>

</div>

> [!NOTE]
> This is a personal project. I published it on GitHub for myself, but if you're
> interested feel free to open an issue, make a PR, or fork it.

Scaffold exists because I want one place to describe how a machine should be put
together the way I want it to be, without pretending one package manager or one
installer is the right answer for every tool on every platform.

It's not trying to replace `Nix`, `winget`, `dnf`, `apt`, `pacman`,
`distrobox`, raw shell commands, or whatever else happens to be the best tool
for a specific job. The whole point is to have a layer where I can say what I
want and let the actual install strategy differ per host, per tool, or honestly
per mood.

I created this project so I could bootstrap any system the way I want it to be.
Solutions like `Nix`, although great for a lot of software, have edge cases and
annoyances that I don't want to deal with anymore.

For example, `Nix` is great for installing stable software that moves slowly.
It's also great for software that is hard to install manually, whether because
it takes a lot of effort to set up well or because its stack is complicated. In
those cases, it makes a lot of sense.

But for a good chunk of the software I use, `Nix` provides little to no value
and sometimes gets in the way. A few examples that come to mind are tools from
[Astral](https://astral.sh/) and [Zed Industries](https://zed.dev/). They make
great software, but it moves quickly, and the nature of `Nix` often means
building it from source. Those projects are _giant_.

That takes a lot of compute, storage, and time. For that kind of software, using
`Nix` is often just a worse experience for me.

On the other hand, `Nix` is still excellent for other projects. `Ghidra` is a
good example: it moves more slowly, its setup and build process can be annoying,
and by the time a new version matters to me it is likely to be cached already.

## Quick start

Scaffold is a Rust workspace, so from a checkout you can do the normal Rust
thing:

```sh
cargo build
cargo run -- --help
```

For local use you can install the CLI from the repo:

```sh
cargo install --path .
scaffold --help
```

Scaffold looks for `scaffold.scm` first, then `catalog.scm`, in the current
directory. You can also pass a catalog explicitly:

```sh
scaffold --catalog /path/to/scaffold.scm check
scaffold --catalog /path/to/scaffold.scm install ripgrep
scaffold --catalog /path/to/scaffold.scm paths --sources
```

`paths --sources` prints the catalog root, extension root, and discovered `.scm`
source/test files, which is useful when debugging split local libraries such as
`extensions/entries/...`.

Useful development commands:

```sh
cargo test --workspace --locked
cargo run -- docs
cargo run -- docs tool
cargo run -- docs --search "ctlg tool"
cargo run -- docs --format json --search "ctlg tool"
cargo run -- docs --source tool
cargo run -- docs --all
cargo run -- docs --output reference.md
cargo run -- docs --output reference.json
bun run docs:dev
bun run docs:check
bun run reference
bun run artifacts:check
bun run vscode:check
bun run check:all
cargo run -- fmt --check crates/scaffold-dsl/src/fixtures/catalog/macro-tools.scm
```

`bun run check:all` runs Rust formatting, clippy, workspace tests, the workspace
build, Biome, fallow dead-code checks, docs site reference/type/build checks,
VS Code reference checks, a non-mutating VS Code compile/WASM binding check,
VSIX package smoke checks, and the WASM smoke test. Browser-hosted VS Code
tests are separate because they require Chromium setup:
`bun run vscode:test:web`.

## Reference docs

`scaffold docs` is a browser for the generated Scheme reference, not a full
reference dump by default. Run it with no arguments to see documentation groups,
pass an exact symbol to open one entry, or use fuzzy search when you only know
part of a name, source path or location, capability, effect, or example:

```sh
scaffold docs
scaffold docs tool
scaffold docs --search "ctlg tool" --limit 10
scaffold docs --search ripgrep
scaffold docs --group Catalog
scaffold docs --source src/dsl/std/catalog/tool.scm
scaffold docs --source src/dsl/std/catalog/tool.scm:16:1
```

Browse output can be rendered as text, Markdown, or JSON. Full reference exports
are explicit; use `--all` for stdout or `--output` for a file:

```sh
scaffold docs tool --format markdown
scaffold docs --search "context read only" --format json
scaffold docs --all
scaffold docs --output reference.md
scaffold docs --output reference.json
```

## Batteries included

Scaffold should offer you everything you need to actually use it. It has a CLI
(ofc), a REPL, eval, formatter, analyzer, tests, generated docs, LSP, VS Code
extension, and MCP server.

I don't want the core idea to be "tiny toy language, good luck". If I'm going to
use a DSL for my own machines, the tooling around it has to be nice enough that
I don't hate opening the file six months later.

The DSL itself is extensible, and you can add your own abstractions for whatever
you need. The Scaffold standard library is mostly primitives, object helpers,
host predicates, catalog builders, checks, and some common installer helpers.
Depending on what you're doing, it should be pretty rare to need your own
abstractions for basic stuff, but you can still write them when your setup gets
weird.

At the same time, I don't want Scaffold to become a giant pile of built-in
opinions about Git, GitHub releases, desktop extensions, dotfiles, language
ecosystems, containers, and every other bespoke workflow. Some common helpers
belong in the tree. A lot of personal workflow glue belongs in your catalog or
your extensions.

### In other words: Scaffold gives you a `Scheme`, not a scheme

## What belongs in Scaffold?

Scaffold includes a CLI, REPL, formatter, analyzer, generated docs, editor
support, an LSP, an MCP server, and a standard library that covers the common
pieces I need to describe real machines.

The boundary I care about is not size. It is whether a feature helps catalogs
stay readable, portable, and honest about how each host should actually be set
up. Runtime behavior, host introspection, object helpers, tests, low-level
mutation primitives, and broadly useful installer helpers belong in Scaffold.
Highly personal workflow glue, one-off installers, and niche opinions usually
belong in a catalog or extension.

The point is still to help you _scaffold_ your setup in the way that is most
convenient and useful to you. If that means using `rpm` for one tool,
`distrobox` with `pacman` for another, `winget` somewhere else, and a raw
install command when that is the best option, so be it. It's your computer, and
it should fit your preferences and needs.

## Why Scheme and not XYZ DSL?

One of the things I really dislike about `Nix` is its bespoke language. Scaffold
still has a DSL, but it is based on something that already exists: `Scheme`.
While developing this privately I initially used `TOML`, and that worked for a
while, but it scaled poorly. Eventually I had multiple `TOML` files over 1000
lines, and at that point it was harder to work through than `Nix`.

## Why not just use XYZ installer

The whole issue is that, for now at least, there is no single installer or
system mutation layer that satisfies all my needs on every platform I use:

- Linux (`NixOS`, "normal" Linux distros, and immutable distros)
- macOS
- Windows

Also:

- You can't use `Nix`, `APT`, or `DNF` natively on Windows. You can
  use some of them under `WSL`, but that is a different setup with its own
  trade-offs.
- On macOS, Scaffold does not bundle an application-manager helper.
- On Windows, Scaffold's bundled application helper targets `winget`.

Scaffold lets those things coexist in one catalog instead of pretending every
machine has the same installer story.

## VS Code extension

The editor extension lives in [editors/vscode](./editors/vscode). It is designed
for browser/web extension use and is backed by the WASM editor engine in
[crates/scaffold-wasm](./crates/scaffold-wasm).

```sh
cd editors/vscode
bun install
bun run compile:check
bun run test:wasm:smoke
```

After changing bundled reference data, source bundles, or the WASM editor
engine, regenerate the committed extension artifacts with:

```sh
cd ../..
bun run reference
bun run artifacts
bun run artifacts:check
```

## License

Scaffold is licensed under the [MIT License](./LICENSE).
