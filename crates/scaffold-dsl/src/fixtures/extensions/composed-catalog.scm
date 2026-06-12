(import
  (rnrs)
  (scaffold catalog)
  (scaffold extensions app flatpak)
  (scaffold extensions app winget)
  (scaffold extensions distro apt)
  (scaffold extensions distro dnf)
  (scaffold extensions distro nix)
  (scaffold extensions distro pacman)
  (scaffold extensions distro rpm)
  (scaffold extensions ecosystem bun))

(catalog
  (tool
    "native"
    (package
      (field
        'platforms
        (arr
          (package/platform-argvs
            'linux
            (arr "dnf")
            (arr (dnf/install-argv "prep-name") (dnf/install-argv "rpm-name")))
          (rpm-ostree/package-platform "rpm-name")
          (dnf/package-platform "rpm-name")
          (pacman/package-platform "pacman-name")
          (apt/package-platform "deb-name")))))
  (nix/profile-package "hello" "nixpkgs#hello")
  (flatpak/app "flatseal" "org.example.Flatseal")
  (bun/global-tool "codex" "@openai/codex" "codex")
  (winget/package "ripgrep" "BurntSushi.ripgrep.MSVC" "rg"))

(moduledoc
  (summary
    "Fixture catalog that composes distro, ecosystem, and target extension helpers."))
