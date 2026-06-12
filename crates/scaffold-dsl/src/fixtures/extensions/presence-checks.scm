(import
  (rnrs)
  (scaffold catalog)
  (scaffold test)
  (scaffold extensions distro apt)
  (scaffold extensions distro dnf)
  (scaffold extensions distro pacman)
  (scaffold extensions distro rpm)
  (scaffold extensions app flatpak))

(define tools
  (arr
    (apt/package "ripgrep")
    (dnf/package "fd-find")
    (pacman/package "bat")
    (rpm-ostree/package "just")
    (flatpak/app "flatseal" "com.github.tchx84.Flatseal")))

(assert/equal
  (arr "dpkg-query" "-W" "{{ package }}")
  (object/ref (vector-ref (object/ref (vector-ref tools 0) 'checks) 0) 'argv))

(assert/equal
  (arr "rpm" "-q" "{{ package }}")
  (object/ref (vector-ref (object/ref (vector-ref tools 1) 'checks) 0) 'argv))

(assert/equal
  (arr "pacman" "-Q" "{{ package }}")
  (object/ref (vector-ref (object/ref (vector-ref tools 2) 'checks) 0) 'argv))

(assert/equal
  (arr "rpm" "-q" "{{ package }}")
  (object/ref (vector-ref (object/ref (vector-ref tools 3) 'checks) 0) 'argv))

(assert/equal
  (arr "flatpak" "info" "com.github.tchx84.Flatseal")
  (object/ref (vector-ref (object/ref (vector-ref tools 4) 'checks) 0) 'argv))

(moduledoc
  (summary "Fixture for bundled extension helpers that attach presence checks."))

(extern-doc tools
  (signature "(tools ...)")
  (summary "Catalog tools vector used to assert generated checks."))
