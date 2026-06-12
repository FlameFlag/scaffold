(library
  (scaffold extensions app winget)
  (export
    winget/install-argv
    winget/upgrade-argv
    winget/uninstall-argv
    winget/list-argv
    winget/package
    winget/package-platform)
  (import (rnrs) (scaffold catalog base) (scaffold extensions support checks))

  (define winget-agreement-flags
    (arr "--accept-package-agreements" "--accept-source-agreements"))

  (doc-next
    (hidden)
    (summary "Build a WinGet argv prefix for an exact package ID command."))

  (define (winget-package-id-argv command package-id)
    (arr "winget" command "--id" package-id "--exact"))

  (doc-next
    (signature "(winget/install-argv package-id flag ...)")
    (summary "Build argv for a noninteractive `winget install --id` command.")
    (param 'package-id "WinGet package identifier.")
    (param 'flag "Additional flags appended after the default silent/agreement flags."))

  (define (winget/install-argv package-id . flags)
    (vector/append
      (winget-package-id-argv "install" package-id)
      (arr "--silent")
      winget-agreement-flags
      (list->vector flags)))

  (doc-next
    (signature "(winget/upgrade-argv package-id flag ...)")
    (summary "Build argv for a noninteractive `winget upgrade --id` command."))

  (define (winget/upgrade-argv package-id . flags)
    (vector/append
      (winget-package-id-argv "upgrade" package-id)
      (arr "--silent")
      winget-agreement-flags
      (list->vector flags)))

  (doc-next
    (signature "(winget/uninstall-argv package-id flag ...)")
    (summary "Build argv for `winget uninstall --id`."))

  (define (winget/uninstall-argv package-id . flags)
    (vector/append
      (winget-package-id-argv "uninstall" package-id)
      (arr "--silent")
      (list->vector flags)))

  (doc-next
    (summary "Build argv for checking an installed package with `winget list --id`."))

  (define (winget/list-argv package-id) (winget-package-id-argv "list" package-id))

  (doc-next
    (hidden)
    (summary "Split WinGet package options into CLI flags and object fields."))

  (define (winget/platform-options options)
    (let loop
      ((rest options) (flags '()) (fields '()))
      (cond
        ((null? rest) (cons (reverse flags) (reverse fields)))
        ((pair? (car rest)) (loop (cdr rest) flags (cons (car rest) fields)))
        (else (loop (cdr rest) (cons (car rest) flags) fields)))))

  (doc-next
    (signature "(winget/package name package-id bin-name field ...)")
    (summary "Create a Windows-only catalog tool installed through WinGet.")
    (param 'name "Catalog tool name.")
    (param 'package-id "WinGet package identifier.")
    (param 'bin-name "Executable exposed by the package.")
    (param 'field "Additional tool fields that override defaults."))

  (define (winget/package name package-id bin-name . fields)
    (apply
      tool
      name
      (package
        (field 'name package-id)
        (field 'install-argv (winget/install-argv "{{ package }}")))
      (field 'platforms (arr 'windows))
      (field 'checks (arr (host/check 'windows (winget/list-argv "{{ package }}"))))
      (field 'bins (arr (bin bin-name)))
      (field
        'uninstall
        (uninstall
          (field
            'commands
            (arr
              (host/uninstall-command 'windows (winget/uninstall-argv "{{ package }}"))))))
      fields))

  (doc-next
    (signature "(winget/package-platform package-id option ...)")
    (summary "Create a Windows package/platform override that requires `winget`.")
    (param 'package-id "WinGet package identifier.")
    (param
      'option
      "Additional install flags or package platform fields. Field values are applied after defaults."))

  (define (winget/package-platform package-id . options)
    (let*
      ((parsed (winget/platform-options options))
        (flags (car parsed))
        (fields (cdr parsed)))
      (apply
        package/platform
        'windows
        (arr "winget")
        (apply winget/install-argv "{{ package }}" flags)
        (field 'name package-id)
        fields)))

  (moduledoc
    (summary "WinGet package helpers for Windows applications.")
    (group "Applications")))
