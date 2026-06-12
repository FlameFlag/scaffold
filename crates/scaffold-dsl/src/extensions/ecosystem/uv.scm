(library
  (scaffold extensions ecosystem uv)
  (export uv/tool-install-argv uv/tool-uninstall-argv uv/tool uv/tool-platform uvx/argv)
  (import (rnrs) (scaffold catalog base))

  (doc-next
    (signature "(uv/tool-install-argv package-name flag ...)")
    (summary "Build argv for installing or upgrading a Python tool with uv.")
    (param 'package-name "Package name passed to `uv tool install`.")
    (param 'flag "Additional `uv tool install` flags placed before the package name.")
    (returns "Vector argv using `uv tool install --upgrade --force`."))

  (define (uv/tool-install-argv package-name . flags)
    (vector/append
      (arr "uv" "tool" "install" "--upgrade" "--force")
      (list->vector flags)
      (arr package-name)))

  (doc-next
    (summary "Build argv for uninstalling a Python tool with uv.")
    (param 'tool-name "Tool name passed to `uv tool uninstall`.")
    (returns "Vector argv for `uv tool uninstall`."))

  (define (uv/tool-uninstall-argv tool-name) (arr "uv" "tool" "uninstall" tool-name))

  (doc-next
    (hidden)
    (summary "Split uv helper options into install flags and object fields."))

  (define (uv/tool-options options)
    (let loop
      ((rest options) (flags '()) (fields '()))
      (cond
        ((null? rest) (cons (reverse flags) (reverse fields)))
        ((pair? (car rest)) (loop (cdr rest) flags (cons (car rest) fields)))
        (else (loop (cdr rest) (cons (car rest) flags) fields)))))

  (doc-next
    (signature "(uv/tool name option ...)")
    (summary "Create a catalog tool installed with `uv tool install`.")
    (param 'name "Python package name and default catalog tool name.")
    (param
      'option
      "Additional `uv tool install` flags or tool fields. Field values are applied after defaults."))

  (define (uv/tool name . options)
    (let*
      ((parsed (uv/tool-options options)) (flags (car parsed)) (fields (cdr parsed)))
      (apply
        tool
        name
        (package
          (field 'name name)
          (field 'install-argv (apply uv/tool-install-argv "{{ package }}" flags)))
        (field 'bins (arr (bin name)))
        (field
          'uninstall
          (uninstall
            (field
              'commands
              (arr (uninstall/command (uv/tool-uninstall-argv "{{ package }}"))))))
        fields)))

  (doc-next
    (signature "(uv/tool-platform predicate package-name option ...)")
    (summary "Create a package/platform override that installs with `uv tool install`.")
    (param 'predicate "Host predicate for this package rule.")
    (param 'package-name "Python package name.")
    (param
      'option
      "Additional `uv tool install` flags or package platform fields. Field values are applied after defaults."))

  (define (uv/tool-platform predicate-value package-name . options)
    (let*
      ((parsed (uv/tool-options options)) (flags (car parsed)) (fields (cdr parsed)))
      (apply
        package/platform
        predicate-value
        (arr "uv")
        (apply uv/tool-install-argv "{{ package }}" flags)
        (field 'name package-name)
        fields)))

  (doc-next
    (signature "(uvx/argv package-name argv ...)")
    (summary "Build argv for running a Python package through `uvx`.")
    (param 'package-name "Package passed to uvx.")
    (param 'argv "Arguments forwarded to the package command."))

  (define (uvx/argv package-name . argv) (apply arr "uvx" package-name argv))

  (moduledoc (summary "uv and uvx helpers for Python ecosystem tools.") (group "uv")))
