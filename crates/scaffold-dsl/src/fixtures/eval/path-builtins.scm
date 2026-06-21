(import (rnrs) (scaffold config) (scaffold path))

(object
  (field 'separator path/separator)
  (field 'single (path/join "vendor"))
  (field 'joined (path/join "vendor" "rg"))
  (field 'normalized (path/normalize (path/join "vendor" ".." "vendor" "rg")))
  (field 'leading-parent (path/normalize (path/join ".." "vendor" ".." "rg")))
  (field 'parent (path/parent (path/join "vendor" "rg" "Cargo.toml")))
  (field 'file-name (path/file-name (path/join "vendor" "rg" "Cargo.toml")))
  (field 'extension (path/extension "archive.tar.gz"))
  (field 'absolute? (path/absolute? path/separator))
  (field 'relative? (path/relative? "vendor")))

(moduledoc (summary "Fixture for Rust-backed lexical path helpers."))
