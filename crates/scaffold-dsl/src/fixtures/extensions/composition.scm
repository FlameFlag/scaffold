(import
  (rnrs)
  (scaffold catalog)
  (scaffold test)
  (scaffold extensions app flatpak)
  (scaffold extensions app winget)
  (scaffold extensions distro apt)
  (scaffold extensions distro dnf)
  (scaffold extensions distro nix)
  (scaffold extensions distro pacman)
  (scaffold extensions distro rpm)
  (scaffold extensions ecosystem bun)
  (scaffold extensions ecosystem cargo)
  (scaffold extensions ecosystem npm)
  (scaffold extensions ecosystem uv))

(define tools
  (object/ref
    (catalog
      (tool
        "native"
        (package
          (field
            'platforms
            (arr
              (rpm-ostree/package-platform "rpm-name")
              (dnf/package-platform "rpm-name")
              (pacman/package-platform "pacman-name")
              (apt/package-platform "deb-name")))))
      (nix/profile-package "hello" "nixpkgs#hello")
      (flatpak/app "flatseal" "org.example.Flatseal")
      (bun/global-tool "codex" "@openai/codex" "codex")
      (winget/package "ripgrep" "BurntSushi.ripgrep.MSVC" "rg"))
    'tools))

(doc-next (summary "Fixture package platform with multiple install commands."))

(define multi-install-platform
  (package/platform-argvs
    'linux
    (arr "dnf")
    (arr (dnf/install-argv "one") (dnf/install-argv "two"))))

(doc-next (summary "Fixture host package platform with multiple install commands."))

(define host-multi-install-platform
  (host/package-platform-argvs
    (arr "toolctl")
    (arr (arr "toolctl" "prepare") (arr "toolctl" "install"))))

(doc-next (summary "Fixture ecosystem package platforms."))

(define ecosystem-platforms
  (arr
    (npm/global-package-platform
      'linux
      "@openai/codex"
      "--registry"
      "https://registry.npmjs.org")
    (bun/global-package-platform 'linux "@openai/codex" "--exact")
    (uv/tool-platform 'linux "ruff" "--python" "3.12")
    (cargo/crate-platform 'linux "cargo-deny" "--features" "native-tls")))

(assert/equal "native" (object/ref (vector-ref tools 0) 'name))

(assert/equal 'package (object/ref (object/ref (vector-ref tools 0) 'action) 'type))

(assert/equal "hello" (object/ref (vector-ref tools 1) 'name))

(assert/equal
  "rg"
  (object/ref (vector-ref (object/ref (vector-ref tools 4) 'bins) 0) 'name))

(assert/equal
  (arr "sudo" "dnf" "install" "-y" "one")
  (vector-ref (object/ref multi-install-platform 'install-argvs) 0))

(assert/equal
  "deb-name"
  (object/ref
    (vector-ref (object/ref (object/ref (vector-ref tools 0) 'action) 'platforms) 3)
    'name))

(assert/equal
  (arr "toolctl" "install")
  (vector-ref (object/ref host-multi-install-platform 'install-argvs) 1))

(assert/equal
  (arr
    "npm"
    "install"
    "--global"
    "--registry"
    "https://registry.npmjs.org"
    "{{ package }}")
  (object/ref (vector-ref ecosystem-platforms 0) 'install-argv))

(assert/equal
  (arr "bun" "add" "-g" "--exact" "{{ package }}")
  (object/ref (vector-ref ecosystem-platforms 1) 'install-argv))

(assert/equal
  (arr "bunx" "@biomejs/biome" "check" ".")
  (bunx/argv "@biomejs/biome" "check" "."))

(assert/equal
  (arr "uv" "tool" "install" "--upgrade" "--force" "--python" "3.12" "{{ package }}")
  (object/ref (vector-ref ecosystem-platforms 2) 'install-argv))

(assert/equal
  (arr
    "cargo"
    "install"
    "{{ package }}"
    "--root"
    "{{ prefix }}"
    "--force"
    "--locked"
    "--features"
    "native-tls")
  (object/ref (vector-ref ecosystem-platforms 3) 'install-argv))

(moduledoc
  (summary "Assertions for composed extension helpers and catalog transforms."))

(extern-doc tools
  (signature "(tools ...)")
  (summary "Tools vector used by extension-composition assertions."))
