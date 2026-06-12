(library
  (scaffold extensions distro nix)
  (export
    nix/experimental-features
    nix/base-argv
    nix/argv
    nix/profile-add-argv
    nix/profile-install-argv
    nix/profile-remove-argv
    nix/profile-package
    nix/profile-platform)
  (import
    (rnrs)
    (scaffold catalog base)
    (scaffold extensions distro nix base)
    (scaffold extensions distro nix profile))

  (moduledoc
    (summary
      "Nix profile package helpers for catalog tools and platform overrides.")
    (group "Nix")))
