(library
  (software-packaging distrobox boxes)
  (export distrobox-arch distrobox-fedora distrobox-debian in-box)
  (import (rnrs) (scaffold catalog) (scaffold extensions target distrobox))

  (doc-next (hidden) (summary "Create a Distrobox environment tool for an image."))

  (define (distrobox-create-tool name image)
    (tool
      name
      (package
        (field 'name name)
        (field
          'install-argv
          (arr "distrobox" "create" "--name" "{{ package }}" "--image" image "--yes")))
      (tool/platforms 'linux)
      (field 'checks (arr (check (arr "distrobox" "list" "--no-color"))))
      (field 'bins (arr (bin "distrobox")))))

  (doc-next (summary "Example Arch Distrobox environment."))

  (define distrobox-arch
    (distrobox-create-tool "scaffold-arch" "quay.io/toolbx/arch-toolbox:latest"))

  (doc-next (summary "Example Fedora Distrobox environment."))

  (define distrobox-fedora
    (distrobox-create-tool
      "scaffold-fedora"
      "registry.fedoraproject.org/fedora-toolbox:latest"))

  (doc-next (summary "Example Debian Distrobox environment."))

  (define distrobox-debian
    (distrobox-create-tool "scaffold-debian" "docker.io/library/debian:stable"))

  (doc-next (summary "Run a tool inside a Distrobox and attach its host dependencies."))

  (define (in-box box box-tool tool-value . dependency-names)
    (tool/inherit
      (distrobox/tool box tool-value)
      (field 'depends (list->vector (cons box-tool dependency-names))))))
