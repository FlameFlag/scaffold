(library
  (software-packaging ecosystems tools)
  (export cargo-deny bun-typescript uv-ruff)
  (import
    (rnrs)
    (scaffold catalog)
    (scaffold extensions ecosystem bun)
    (scaffold extensions ecosystem cargo)
    (scaffold extensions ecosystem uv))

  (doc-next (summary "Example Cargo crate tool."))

  (define cargo-deny
    (cargo/crate-tool
      "cargo-deny"
      "cargo-deny"
      "cargo-deny"
      (meta (description "Example Rust tool installed from crates.io."))))

  (doc-next (summary "Example Bun TypeScript tool."))

  (define bun-typescript
    (bun/global-tool
      "bun-typescript"
      "typescript"
      "tsc"
      (depends "arch-nix-profile-basics")
      (meta (description "Example JavaScript tool installed globally with Bun."))))

  (doc-next (summary "Example uv Ruff tool."))

  (define uv-ruff
    (uv/tool
      "ruff"
      (depends "debian-nix-profile-basics")
      (field 'bins (arr (bin "ruff")))
      (meta (description "Example Python tool installed with uv.")))))
