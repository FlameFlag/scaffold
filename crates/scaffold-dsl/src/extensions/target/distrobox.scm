(library
  (scaffold extensions target distrobox)
  (export
    distrobox/enter-argv
    distrobox/host-exec-argv
    distrobox/package
    distrobox/tool)
  (import (rnrs) (scaffold catalog base))

  (doc-next
    (summary "Wrap argv so it runs through `distrobox enter <box> --`.")
    (param 'box "Distrobox container name.")
    (param 'argv "Command vector to run inside the container.")
    (returns "Wrapped argv vector."))

  (define (distrobox/enter-argv box argv)
    (vector/append (arr "distrobox" "enter" box "--") argv))

  (doc-next
    (summary "Wrap argv so a Distrobox process runs it on the host.")
    (param 'argv "Command vector to run through `distrobox-host-exec`."))

  (define (distrobox/host-exec-argv argv)
    (vector/append (arr "distrobox-host-exec") argv))

  (doc-next
    (summary "Rewrite a package action so install commands run inside a Distrobox.")
    (param 'box "Distrobox container name.")
    (param 'package-action "Package action to rewrite.")
    (returns "A package action with install argv fields wrapped for the container."))

  (define (distrobox/package box package-action)
    (package/map-install-argvs
      package-action
      (lambda (argv) (distrobox/enter-argv box argv))))

  (doc-next
    (summary
      "Rewrite a tool so install, check, and uninstall commands run inside a Distrobox.")
    (param 'box "Distrobox container name.")
    (param 'tool-value "Tool object to rewrite.")
    (returns "A tool object with install, check, and uninstall argv wrapped."))

  (define (distrobox/tool box tool-value)
    (tool/map-uninstall-command-argvs
      (tool/map-check-argvs
        (tool/map-package-install-argvs
          tool-value
          (lambda (argv) (distrobox/enter-argv box argv)))
        (lambda (argv) (distrobox/enter-argv box argv)))
      (lambda (argv) (distrobox/enter-argv box argv))))

  (moduledoc
    (summary
      "Helpers for adapting package actions, checks, and uninstall commands to run inside a Distrobox container.")
    (group "Targets")))
