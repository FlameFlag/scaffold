(library
  (software-packaging distrobox native)
  (export arch-native-packages fedora-native-packages debian-native-packages)
  (import
    (rnrs)
    (scaffold catalog)
    (scaffold extensions distro apt)
    (scaffold extensions distro dnf)
    (scaffold extensions distro pacman)
    (software-packaging distrobox boxes))

  (doc-next (summary "Example native package set for an Arch Distrobox."))

  (define arch-native-packages
    (in-box
      "scaffold-arch"
      "scaffold-arch"
      (tool
        "arch-native-packages"
        (package
          (field 'name "arch-native-packages")
          (field
            'install-argv
            (pacman/install-argv "base-devel" "curl" "jq" "nix" "code")))
        (field 'bins (arr (bin "pacman"))))))

  (doc-next (summary "Example native package set for a Fedora Distrobox."))

  (define fedora-native-packages
    (in-box
      "scaffold-fedora"
      "scaffold-fedora"
      (tool
        "fedora-native-packages"
        (package
          (field 'name "fedora-native-packages")
          (field 'install-argv (dnf/install-argv "gcc" "make" "curl" "jq" "nix" "code")))
        (field 'bins (arr (bin "dnf"))))))

  (doc-next (summary "Example native package set for a Debian Distrobox."))

  (define debian-native-packages
    (in-box
      "scaffold-debian"
      "scaffold-debian"
      (tool
        "debian-native-packages"
        (package
          (field 'name "debian-native-packages")
          (field
            'install-argvs
            (arr
              (arr "sudo" "apt-get" "update")
              (apt-get/install-argv "build-essential" "curl" "jq" "nix-bin" "code"))))
        (field 'bins (arr (bin "apt-get")))))))
