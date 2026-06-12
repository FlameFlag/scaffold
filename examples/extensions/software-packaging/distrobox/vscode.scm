(library
  (software-packaging distrobox vscode)
  (export arch-vscode fedora-vscode debian-vscode)
  (import
    (rnrs)
    (scaffold catalog)
    (scaffold extensions distro apt)
    (scaffold extensions distro dnf)
    (scaffold extensions distro pacman)
    (software-packaging distrobox boxes))

  (doc-next (summary "Example Arch Distrobox VS Code tool."))

  (define arch-vscode
    (in-box
      "scaffold-arch"
      "scaffold-arch"
      (pacman/package-tool "arch-vscode" "code" "code")
      "arch-native-packages"))

  (doc-next (summary "Example Fedora Distrobox VS Code tool."))

  (define fedora-vscode
    (in-box
      "scaffold-fedora"
      "scaffold-fedora"
      (dnf/package-tool "fedora-vscode" "code" "code")
      "fedora-native-packages"))

  (doc-next (summary "Example Debian Distrobox VS Code tool."))

  (define debian-vscode
    (in-box
      "scaffold-debian"
      "scaffold-debian"
      (apt/package-tool "debian-vscode" "code" "code")
      "debian-native-packages")))
