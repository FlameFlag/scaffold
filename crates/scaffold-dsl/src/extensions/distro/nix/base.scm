(library
  (scaffold extensions distro nix base)
  (export nix/experimental-features nix/base-argv nix/argv)
  (import (rnrs) (scaffold catalog base))

  (doc-next
    (summary "Default Nix experimental feature string used by this module.")
    (returns "`\"nix-command flakes\"`."))

  (define nix/experimental-features "nix-command flakes")

  (doc-next
    (summary "Base argv prefix for all Nix helpers.")
    (returns "Vector argv for `nix --extra-experimental-features ...`."))

  (define nix/base-argv
    (arr "nix" "--extra-experimental-features" nix/experimental-features))

  (doc-next
    (signature "(nix/argv argv ...)")
    (summary "Append Nix subcommands and flags to the shared Nix argv prefix.")
    (param 'argv "Subcommand names, flags, and arguments.")
    (returns "Vector argv beginning with `nix --extra-experimental-features`."))

  (define (nix/argv . argv) (arr/append-list nix/base-argv argv))

  (moduledoc (summary "Shared Nix argv prefix helpers.") (group "Nix")))
