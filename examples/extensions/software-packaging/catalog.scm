(library
  (software-packaging catalog)
  (export software-packaging/catalog)
  (import
    (rnrs)
    (scaffold catalog)
    (software-packaging desktop vscode)
    (software-packaging distrobox boxes)
    (software-packaging distrobox git)
    (software-packaging distrobox native)
    (software-packaging distrobox nix)
    (software-packaging distrobox vscode)
    (software-packaging ecosystems tools)
    (software-packaging source git))

  (doc-next (summary "Build the example software packaging catalog."))

  (define (software-packaging/catalog)
    (catalog
      custom-git-linux
      custom-git-macos
      mingw-w64-toolchain
      custom-git-windows
      vscode
      distrobox-arch
      distrobox-fedora
      distrobox-debian
      arch-native-packages
      fedora-native-packages
      debian-native-packages
      arch-nix-profile-basics
      fedora-nix-profile-basics
      debian-nix-profile-basics
      arch-custom-git
      fedora-custom-git
      debian-custom-git
      arch-vscode
      fedora-vscode
      debian-vscode
      cargo-deny
      bun-typescript
      uv-ruff)))
