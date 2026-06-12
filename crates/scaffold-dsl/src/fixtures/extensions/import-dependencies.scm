(import
  (rnrs)
  (scaffold catalog)
  (scaffold test)
  (scaffold extensions app flatpak)
  (scaffold extensions distro apt))

(define apt-demo (apt/package "ripgrep"))

(define flatpak-demo (flatpak/app "flatseal" "com.github.tchx84.Flatseal"))

(doc-next (summary "Fixture Flatpak platform override."))

(define flatpak-platform (flatpak/package-platform "com.github.tchx84.Flatseal"))

(assert/equal
  (arr "dpkg-query" "-W" "{{ package }}")
  (object/ref (vector-ref (object/ref apt-demo 'checks) 0) 'argv))

(assert/equal
  (arr "flatpak" "info" "com.github.tchx84.Flatseal")
  (object/ref (vector-ref (object/ref flatpak-demo 'checks) 0) 'argv))

(assert/equal "com.github.tchx84.Flatseal" (object/ref flatpak-platform 'name))

(assert/equal "flatpak" (vector-ref (object/ref flatpak-platform 'requires-commands) 0))

(assert/equal
  (arr "flatpak" "install" "--assumeyes" "--noninteractive" "flathub" "{{ package }}")
  (object/ref flatpak-platform 'install-argv))

(moduledoc (summary "Fixture for bundled extension dependency imports."))

(extern-doc apt-demo
  (signature "(apt-demo ...)")
  (summary "APT demo tool used to assert bundled dependency loading."))

(extern-doc flatpak-demo
  (signature "(flatpak-demo ...)")
  (summary "Flatpak demo tool used to assert bundled dependency loading."))
