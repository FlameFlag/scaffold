(library
  (scaffold extensions ecosystem npm)
  (export
    npm/global-install-argv
    npm/global-uninstall-argv
    npm/global-tool
    npm/global-package-platform
    npx/argv)
  (import (rnrs) (scaffold catalog base))

  (doc-next
    (signature "(npm/global-install-argv package-name flag ...)")
    (summary "Build argv for globally installing an npm package.")
    (param 'package-name "npm package name.")
    (param 'flag "Additional npm install flags placed before the package name.")
    (returns "Vector argv for `npm install --global`."))

  (doc-next
    (hidden)
    (summary "Build npm global install argv from an existing flag list."))

  (define (npm/global-install-argv-list package-name flags)
    (vector/append
      (arr/append-list (arr "npm" "install" "--global") flags)
      (arr package-name)))

  (define (npm/global-install-argv package-name . flags)
    (npm/global-install-argv-list package-name flags))

  (doc-next
    (summary "Build argv for uninstalling a global npm package.")
    (param 'package-name "npm package name.")
    (returns "Vector argv for `npm uninstall --global`."))

  (define (npm/global-uninstall-argv package-name)
    (arr "npm" "uninstall" "--global" package-name))

  (doc-next
    (signature "(npm/global-tool name package-name bin-name option ...)")
    (summary "Create a catalog tool installed globally with npm.")
    (param 'name "Catalog tool name.")
    (param 'package-name "npm package to install.")
    (param 'bin-name "Executable exposed by the package.")
    (param
      'option
      "Additional npm install flags or tool fields. Field values are applied after defaults."))

  (define (npm/global-tool name package-name bin-name . options)
    (call-with-split-fields
      options
      (lambda (flags fields)
        (object/merge
          (tool
            name
            (package
              (field 'name package-name)
              (field
                'install-argv
                (npm/global-install-argv-list "{{ package }}" flags)))
            (field 'bins (arr (bin bin-name)))
            (field
              'uninstall
              (uninstall
                (field
                  'commands
                  (arr
                    (uninstall/command (npm/global-uninstall-argv "{{ package }}")))))))
          fields))))

  (doc-next
    (signature "(npm/global-package-platform predicate package-name option ...)")
    (summary "Create a package/platform override that installs with global npm.")
    (param 'predicate "Host predicate for this package rule.")
    (param 'package-name "npm package name.")
    (param
      'option
      "Additional npm install flags or package platform fields. Field values are applied after defaults."))

  (define (npm/global-package-platform predicate-value package-name . options)
    (call-with-split-fields
      options
      (lambda (flags fields)
        (object/merge
          (package/platform
            predicate-value
            (arr "npm")
            (npm/global-install-argv-list "{{ package }}" flags)
            (field 'name package-name))
          fields))))

  (doc-next
    (signature "(npx/argv package-name argv ...)")
    (summary "Build argv for running a package through `npx`.")
    (param 'package-name "npm package passed to npx.")
    (param 'argv "Arguments forwarded to the package command."))

  (define (npx/argv package-name . argv)
    (arr/append-list (arr "npx" package-name) argv))

  (moduledoc
    (summary "npm and npx helpers for JavaScript ecosystem tools.")
    (group "npm")))
