(library
  (examples targets distrobox)
  (export dev-fedora fedora-jq distrobox/tools tool/in-distrobox)
  (import (rnrs) (scaffold catalog))

  (moduledoc (summary "Distrobox target wrapping examples.") (group "Examples"))

  (doc-next (hidden) (summary "Prepend a Distrobox enter prefix to an argv vector."))

  (define (distrobox/argv box argv)
    (vector/append (arr "distrobox" "enter" box "--") argv))

  (doc-next
    (summary "Derive a tool that runs package installs and checks inside a Distrobox."))

  (define (tool/in-distrobox box box-tool tool-value)
    (tool/map-check-argvs
      (tool/map-package-install-argvs
        (tool/inherit
          tool-value
          (field 'name (string-append box "-" (object/ref tool-value 'name)))
          (depends box-tool))
        (lambda (argv) (distrobox/argv box argv)))
      (lambda (argv) (distrobox/argv box argv))))

  (doc-next (summary "Fedora development Distrobox environment."))

  (define dev-fedora
    (tool
      "dev-fedora"
      (package
        (field 'name "dev-fedora")
        (field
          'install-argv
          (arr
            "distrobox"
            "create"
            "--name"
            "{{ package }}"
            "--image"
            "registry.fedoraproject.org/fedora-toolbox:latest"
            "--yes")))
      (tool/platforms 'linux)
      (field 'checks (arr (check (arr "distrobox" "list" "--no-color"))))
      (field 'bins (arr (bin "distrobox")))
      (meta
        (description "Example Fedora toolbox used as a package target.")
        (tags "container" "distrobox")
        (main-program "distrobox"))))

  (doc-next (summary "jq package definition before Distrobox wrapping."))

  (define jq
    (tool
      "jq"
      (package (field 'install-argv (arr "sudo" "dnf" "install" "-y" "jq")))
      (field 'bins (arr (bin/version "jq" "--version")))
      (field 'checks (arr (check (arr "jq" "--version"))))
      (meta
        (description "JSON processor installed inside the Fedora Distrobox.")
        (home-page "https://jqlang.github.io/jq/")
        (license "MIT")
        (tags "json" "cli")
        (main-program "jq"))))

  (doc-next (summary "jq derived to install and check inside the Fedora Distrobox."))

  (define fedora-jq (tool/in-distrobox "dev-fedora" "dev-fedora" jq))

  (doc-next (summary "Return Distrobox target examples."))

  (define (distrobox/tools) (list dev-fedora fedora-jq)))
