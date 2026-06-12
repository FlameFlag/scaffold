(library
  (software-packaging distrobox nix)
  (export arch-nix-profile-basics fedora-nix-profile-basics debian-nix-profile-basics)
  (import
    (rnrs)
    (scaffold catalog)
    (scaffold extensions distro nix)
    (software-packaging distrobox boxes))

  (doc-next (hidden) (summary "Build a Distrobox-wrapped Nix profile example tool."))

  (define (nix-profile-basics name box box-tool native-packages-tool)
    (in-box
      box
      box-tool
      (tool
        name
        (package
          (field 'name name)
          (field
            'install-argv
            (nix/profile-add-argv "nixpkgs#ripgrep" "nixpkgs#fd" "nixpkgs#jq")))
        (field 'bins (arr (bin "rg") (bin "fd") (bin "jq"))))
      native-packages-tool))

  (doc-next (summary "Example Arch Distrobox Nix profile tool."))

  (define arch-nix-profile-basics
    (nix-profile-basics
      "arch-nix-profile-basics"
      "scaffold-arch"
      "scaffold-arch"
      "arch-native-packages"))

  (doc-next (summary "Example Fedora Distrobox Nix profile tool."))

  (define fedora-nix-profile-basics
    (nix-profile-basics
      "fedora-nix-profile-basics"
      "scaffold-fedora"
      "scaffold-fedora"
      "fedora-native-packages"))

  (doc-next (summary "Example Debian Distrobox Nix profile tool."))

  (define debian-nix-profile-basics
    (nix-profile-basics
      "debian-nix-profile-basics"
      "scaffold-debian"
      "scaffold-debian"
      "debian-native-packages")))
