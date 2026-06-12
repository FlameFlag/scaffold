(import
  (rnrs)
  (scaffold catalog)
  (scaffold test)
  (scaffold extensions app flatpak)
  (scaffold extensions distro apt)
  (scaffold extensions distro dnf)
  (scaffold extensions distro pacman)
  (scaffold extensions distro rpm)
  (scaffold extensions ecosystem bun)
  (scaffold extensions ecosystem cargo)
  (scaffold extensions ecosystem npm)
  (scaffold extensions ecosystem uv))

(doc-next (summary "Return the first uninstall command argv from a tool object."))

(define (first-uninstall-argv tool-value)
  (object/ref
    (vector-ref (object/ref (object/ref tool-value 'uninstall) 'commands) 0)
    'argv))

(define apt-demo (apt/package "ripgrep"))

(doc-next (summary "Fixture DNF package tool."))

(define dnf-demo (dnf/package "ripgrep"))

(doc-next (summary "Fixture Pacman package tool."))

(define pacman-demo (pacman/package "ripgrep"))

(doc-next (summary "Fixture rpm-ostree package tool."))

(define rpm-ostree-demo (rpm-ostree/package "ripgrep"))

(doc-next
  (summary "Fixture rpm-ostree package tool with distinct tool and package names."))

(define rpm-ostree-tool-demo (rpm-ostree/package-tool "rg-rpm-ostree" "ripgrep" "rg"))

(define flatpak-demo (flatpak/app "flatseal" "com.github.tchx84.Flatseal"))

(doc-next (summary "Fixture Bun global tool."))

(define bun-demo (bun/global-tool "codex-bun" "@openai/codex" "codex"))

(doc-next (summary "Fixture Cargo-installed tool."))

(define cargo-demo (cargo/tool "ripgrep" "vendor/ripgrep"))

(doc-next (summary "Fixture Cargo crate package tool."))

(define cargo-crate-demo (cargo/crate-tool "cargo-deny" "cargo-deny" "cargo-deny"))

(doc-next (summary "Fixture npm global tool."))

(define npm-demo (npm/global-tool "codex" "@openai/codex" "codex"))

(doc-next (summary "Fixture uv-installed tool."))

(define uv-demo (uv/tool "ruff"))

(assert/equal 'linux (vector-ref (object/ref apt-demo 'platforms) 0))

(assert/equal
  (arr "sudo" "apt-get" "remove" "-y" "{{ package }}")
  (first-uninstall-argv apt-demo))

(assert/equal
  (arr "sudo" "dnf" "remove" "-y" "{{ package }}")
  (first-uninstall-argv dnf-demo))

(assert/equal
  (arr "sudo" "pacman" "-R" "--noconfirm" "{{ package }}")
  (first-uninstall-argv pacman-demo))

(assert/equal
  (arr "sudo" "rpm-ostree" "uninstall" "-y" "{{ package }}")
  (first-uninstall-argv rpm-ostree-demo))

(assert/equal "rg-rpm-ostree" (object/ref rpm-ostree-tool-demo 'name))

(assert/equal "ripgrep" (object/ref (object/ref rpm-ostree-tool-demo 'action) 'name))

(assert/equal
  "rg"
  (object/ref (vector-ref (object/ref rpm-ostree-tool-demo 'bins) 0) 'name))

(assert/equal
  (arr "flatpak" "uninstall" "--assumeyes" "--noninteractive" "{{ package }}")
  (first-uninstall-argv flatpak-demo))

(assert/equal (arr "bun" "remove" "-g" "{{ package }}") (first-uninstall-argv bun-demo))

(assert/equal
  (arr "cargo" "uninstall" "--root" "{{ prefix }}" "ripgrep")
  (first-uninstall-argv cargo-demo))

(assert/equal
  (arr "cargo" "install" "{{ package }}" "--root" "{{ prefix }}" "--force" "--locked")
  (object/ref (object/ref cargo-crate-demo 'action) 'install-argv))

(assert/equal
  (arr "cargo" "uninstall" "--root" "{{ prefix }}" "{{ package }}")
  (first-uninstall-argv cargo-crate-demo))

(assert/equal
  (arr "npm" "uninstall" "--global" "{{ package }}")
  (first-uninstall-argv npm-demo))

(assert/equal
  (arr "uv" "tool" "uninstall" "{{ package }}")
  (first-uninstall-argv uv-demo))

(moduledoc (summary "Fixture for package lifecycle helper uninstall metadata."))
