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

  (doc-next
    (hidden)
    (summary "Build uv tool install argv from an existing flag list."))

  (define (uv/tool-install-argv-list package-name flags)
    (vector/append
      (arr/append-list (arr "uv" "tool" "install" "--upgrade" "--force") flags)
      (arr package-name)))

  (define (uv/tool-install-argv package-name . flags)
    (uv/tool-install-argv-list package-name flags))

  (doc-next
    (summary "Build argv for uninstalling a Python tool with uv.")
    (param 'tool-name "Tool name passed to `uv tool uninstall`.")
    (returns "Vector argv for `uv tool uninstall`."))

  (define (uv/tool-uninstall-argv tool-name) (arr "uv" "tool" "uninstall" tool-name))

  (doc-next
    (signature "(uv/tool name option ...)")
    (summary "Create a catalog tool installed with `uv tool install`.")
    (param 'name "Python package name and default catalog tool name.")
    (param
      'option
      "Additional `uv tool install` flags or tool fields. Field values are applied after defaults."))

  (define (uv/tool name . options)
    (call-with-split-fields
      options
      (lambda (flags fields)
        (object/merge
          (tool
            name
            (package
              (field 'name name)
              (field 'install-argv (uv/tool-install-argv-list "{{ package }}" flags)))
            (field 'bins (arr (bin name)))
            (field
              'uninstall
              (uninstall
                (field
                  'commands
                  (arr (uninstall/command (uv/tool-uninstall-argv "{{ package }}")))))))
          fields))))

  (doc-next
    (signature "(uv/tool-platform predicate package-name option ...)")
    (summary "Create a package/platform override that installs with `uv tool install`.")
    (param 'predicate "Host predicate for this package rule.")
    (param 'package-name "Python package name.")
    (param
      'option
      "Additional `uv tool install` flags or package platform fields. Field values are applied after defaults."))

  (define (uv/tool-platform predicate-value package-name . options)
    (call-with-split-fields
      options
      (lambda (flags fields)
        (object/merge
          (package/platform
            predicate-value
            (arr "uv")
            (uv/tool-install-argv-list "{{ package }}" flags)
            (field 'name package-name))
          fields))))

  (doc-next
    (signature "(uvx/argv package-name argv ...)")
    (summary "Build argv for running a Python package through `uvx`.")
    (param 'package-name "Package passed to uvx.")
    (param 'argv "Arguments forwarded to the package command."))

  (define (uvx/argv package-name . argv)
    (arr/append-list (arr "uvx" package-name) argv))

  (moduledoc (summary "uv and uvx helpers for Python ecosystem tools.") (group "uv")))
