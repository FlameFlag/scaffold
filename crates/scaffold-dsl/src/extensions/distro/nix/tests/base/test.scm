(import (rnrs) (scaffold catalog) (scaffold test) (scaffold extensions distro nix))

(assert/equal
  (arr "nix" "--extra-experimental-features" "nix-command flakes" "eval")
  (nix/argv "eval"))

(assert/equal (arr "nix" "eval") (arr/append-list (arr "nix") (list "eval")))

(assert/equal (arr "nix" "eval") (arr/prepend-list (list "nix") (arr "eval")))

(moduledoc (summary "Regression tests for shared Nix argv helpers."))
