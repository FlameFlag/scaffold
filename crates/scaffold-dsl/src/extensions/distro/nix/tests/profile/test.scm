(import (rnrs) (scaffold catalog) (scaffold test) (scaffold extensions distro nix))

(assert/equal
  (arr
    "nix"
    "--extra-experimental-features"
    "nix-command flakes"
    "profile"
    "add"
    "{{ package }}")
  (nix/profile-add-argv))

(assert/equal (nix/profile-add-argv) (nix/profile-install-argv))

(assert/equal
  (arr
    "nix"
    "--extra-experimental-features"
    "nix-command flakes"
    "profile"
    "add"
    "nixpkgs#hello"
    "nixpkgs#jq")
  (nix/profile-add-argv "nixpkgs#hello" "nixpkgs#jq"))

(assert/equal
  (arr
    "nix"
    "--extra-experimental-features"
    "nix-command flakes"
    "profile"
    "remove"
    "hello")
  (nix/profile-remove-argv "hello"))

(doc-next
  (signature "(hello-tool ...)")
  (summary "Nix profile package fixture used by command-builder assertions."))

(define hello-tool (nix/profile-package "hello" "nixpkgs#hello"))

(doc-next
  (signature "(hello-platform ...)")
  (summary "Nix profile platform fixture used by command-builder assertions."))

(define hello-platform (nix/profile-platform 'linux "nixpkgs#hello"))

(assert/equal "hello" (object/ref hello-tool 'name))

(assert/equal "nixpkgs#hello" (object/ref (object/ref hello-tool 'action) 'name))

(assert/equal
  (nix/profile-remove-argv "hello")
  (object/ref
    (vector-ref (object/ref (object/ref hello-tool 'uninstall) 'commands) 0)
    'argv))

(assert/equal "nixpkgs#hello" (object/ref hello-platform 'name))

(assert/equal
  (nix/profile-add-argv "{{ package }}")
  (object/ref hello-platform 'install-argv))

(moduledoc (summary "Regression tests for Nix profile helpers."))
