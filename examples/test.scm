(import
  (rnrs)
  (scaffold catalog)
  (scaffold test)
  (examples targets distrobox)
  (examples tools builds)
  (examples tools desktop)
  (examples tools ecosystem)
  (examples tools packages)
  (examples tools prerequisites))

(doc-next (summary "Return the action object from a tool value."))

(define (action tool-value) (object/ref tool-value 'action))

(doc-next (summary "Return the install argv from a package action."))

(define (install-argv tool-value) (object/ref (action tool-value) 'install-argv))

(doc-next (summary "Return the package/platform vector from a package action."))

(define (package-platforms tool-value) (object/ref (action tool-value) 'platforms))

(doc-next (summary "Return the first check argv from a tool value."))

(define (first-check-argv tool-value)
  (object/ref (vector-ref (object/ref tool-value 'checks) 0) 'argv))

(assert/equal 'required (object/ref (action git) 'type))

(assert/equal (arr "git" "--version") (first-check-argv git))

(assert/equal
  (arr "sudo" "apt-get" "install" "-y" "{{ package }}")
  (object/ref (vector-ref (package-platforms ripgrep) 0) 'install-argv))

(assert/equal
  (arr
    "winget"
    "install"
    "--id"
    "{{ package }}"
    "--exact"
    "--silent"
    "--accept-package-agreements"
    "--accept-source-agreements")
  (object/ref (vector-ref (package-platforms ripgrep) 3) 'install-argv))

(assert/equal
  (arr "flatpak" "install" "--assumeyes" "--noninteractive" "flathub" "{{ package }}")
  (object/ref (vector-ref (package-platforms vscode) 0) 'install-argv))

(assert/equal (arr "bun" "add" "-g" "{{ package }}") (install-argv prettier))

(assert/equal
  (arr "uv" "tool" "install" "--upgrade" "--force" "{{ package }}")
  (install-argv ruff))

(assert/equal
  (arr "distrobox" "enter" "dev-fedora" "--" "sudo" "dnf" "install" "-y" "jq")
  (install-argv fedora-jq))

(assert/equal
  (arr "distrobox" "enter" "dev-fedora" "--" "jq" "--version")
  (first-check-argv fedora-jq))

(assert/equal "dev-fedora" (vector-ref (object/ref fedora-jq 'depends) 0))

(assert/equal
  (arr "sh" "-lc" "cc hello.c -o {{ bin_dir }}/hello-example")
  (vector-ref (object/ref (action hello-example) 'argvs) 0))
