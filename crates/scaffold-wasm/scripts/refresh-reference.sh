#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cargo run --manifest-path "$repo_root/Cargo.toml" -p scaffold -- docs --format json \
  > "$repo_root/crates/scaffold-wasm/src/reference.json"
node -e 'const fs = require("fs"); const path = process.argv[1]; fs.writeFileSync(path.replace(/\.json$/, ".min.json"), JSON.stringify(JSON.parse(fs.readFileSync(path, "utf8"))));' \
  "$repo_root/crates/scaffold-wasm/src/reference.json"
