(library
  (scaffold extensions ecosystem bun)
  (export
    bun/global-install-argv
    bun/global-uninstall-argv
    bun/global-tool
    bun/global-package-platform
    bunx/argv)
  (import (rnrs) (scaffold catalog base))

  (doc-next
    (signature "(bun/global-install-argv package-name flag ...)")
    (summary "Build argv for globally installing a package with Bun.")
    (param 'package-name "Package name passed to `bun add -g`.")
    (param 'flag "Additional Bun add flags placed before the package name.")
    (returns "Vector argv for `bun add -g`."))

  (define (bun/global-install-argv package-name . flags)
    (vector/append (arr "bun" "add" "-g") (list->vector flags) (arr package-name)))

  (doc-next
    (summary "Build argv for removing a globally installed Bun package.")
    (param 'package-name "Package name passed to `bun remove -g`.")
    (returns "Vector argv for `bun remove -g`."))

  (define (bun/global-uninstall-argv package-name)
    (arr "bun" "remove" "-g" package-name))

  (doc-next
    (hidden)
    (summary "Split Bun helper options into install flags and object fields."))

  (define (bun/global-options options)
    (let loop
      ((rest options) (flags '()) (fields '()))
      (cond
        ((null? rest) (cons (reverse flags) (reverse fields)))
        ((pair? (car rest)) (loop (cdr rest) flags (cons (car rest) fields)))
        (else (loop (cdr rest) (cons (car rest) flags) fields)))))

  (doc-next
    (signature "(bun/global-tool name package-name bin-name option ...)")
    (summary "Create a catalog tool installed globally with Bun.")
    (param 'name "Catalog tool name.")
    (param 'package-name "Package to install.")
    (param 'bin-name "Executable exposed by the package.")
    (param
      'option
      "Additional Bun add flags or tool fields. Field values are applied after defaults."))

  (define (bun/global-tool name package-name bin-name . options)
    (let*
      ((parsed (bun/global-options options)) (flags (car parsed)) (fields (cdr parsed)))
      (apply
        tool
        name
        (package
          (field 'name package-name)
          (field 'install-argv (apply bun/global-install-argv "{{ package }}" flags)))
        (field 'bins (arr (bin bin-name)))
        (field
          'uninstall
          (uninstall
            (field
              'commands
              (arr (uninstall/command (bun/global-uninstall-argv "{{ package }}"))))))
        fields)))

  (doc-next
    (signature "(bun/global-package-platform predicate package-name option ...)")
    (summary "Create a package/platform override that installs with global Bun.")
    (param 'predicate "Host predicate for this package rule.")
    (param 'package-name "Package name passed to `bun add -g`.")
    (param
      'option
      "Additional Bun add flags or package platform fields. Field values are applied after defaults."))

  (define (bun/global-package-platform predicate-value package-name . options)
    (let*
      ((parsed (bun/global-options options)) (flags (car parsed)) (fields (cdr parsed)))
      (apply
        package/platform
        predicate-value
        (arr "bun")
        (apply bun/global-install-argv "{{ package }}" flags)
        (field 'name package-name)
        fields)))

  (doc-next
    (signature "(bunx/argv package-name argv ...)")
    (summary "Build argv for running a package through `bunx`.")
    (param 'package-name "Package passed to bunx.")
    (param 'argv "Arguments forwarded to the package command."))

  (define (bunx/argv package-name . argv) (apply arr "bunx" package-name argv))

  (moduledoc
    (summary "Bun package and bunx helpers for JavaScript ecosystem tools.")
    (group "Bun")))
