(library
  (scaffold catalog transform)
  (export
    package/map-install-argvs
    tool/map-package-install-argvs
    tool/map-check-argvs
    uninstall/map-command-argvs
    tool/map-uninstall-command-argvs
    install-argv/prepend
    argv/append)
  (import (rnrs) (scaffold config))

  (doc-next
    (hidden)
    (summary "Transform install argv fields inside one package platform object."))

  (define (map-package-platform-install-argvs platform proc)
    (object/map-vector-field
      (object/update-field platform 'install-argv proc)
      'install-argvs
      proc))

  (doc-next
    (summary "Apply a transformation to all install argv fields in a package action.")
    (param
      'package-action
      "Package action that may contain `install-argv`, `install-argvs`, or platform overrides.")
    (param 'proc "Procedure that receives and returns an argv vector."))

  (define (package/map-install-argvs package-action proc)
    (let
      ((with-install-argvs
         (object/map-vector-field
           (object/update-field package-action 'install-argv proc)
           'install-argvs
           proc)))
      (object/map-vector-field
        with-install-argvs
        'platforms
        (lambda (platform) (map-package-platform-install-argvs platform proc)))))

  (doc-next (summary "Transform package install argv fields inside a tool's action."))

  (define (tool/map-package-install-argvs tool-value proc)
    (object/replace-field
      tool-value
      'action
      (package/map-install-argvs (object/ref tool-value 'action) proc)))

  (doc-next (summary "Transform each check argv vector in a tool."))

  (define (tool/map-check-argvs tool-value proc)
    (object/map-vector-field
      tool-value
      'checks
      (lambda (check-value)
        (object/update-field check-value 'argv proc))))

  (doc-next
    (summary "Transform each uninstall command argv vector.")
    (param 'uninstall-value "Uninstall metadata object.")
    (param 'proc "Procedure that receives and returns an argv vector."))

  (define (uninstall/map-command-argvs uninstall-value proc)
    (object/map-vector-field
      uninstall-value
      'commands
      (lambda (command-value)
        (object/update-field command-value 'argv proc))))

  (doc-next (summary "Transform uninstall command argv fields inside a tool."))

  (define (tool/map-uninstall-command-argvs tool-value proc)
    (object/update-field
      tool-value
      'uninstall
      (lambda (uninstall-value) (uninstall/map-command-argvs uninstall-value proc))))

  (doc-next
    (signature "(install-argv/prepend obj argv ...)")
    (summary "Prepend argv parts to an object's `install-argv` vector."))

  (define (install-argv/prepend obj . argv)
    (object/replace-field
      obj
      'install-argv
      (arr/prepend-list argv (object/ref obj 'install-argv (arr)))))

  (doc-next (summary "Append a Scheme list of argv items to an argv vector."))

  (define (argv/append prefix items) (arr/append-list prefix items))

  (moduledoc
    (summary "Catalog object and argv transformation helpers.")
    (group "Transformations")))
