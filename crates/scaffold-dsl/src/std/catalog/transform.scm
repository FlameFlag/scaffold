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
    (let
      ((with-install-argv
         (if (object/has-field? platform 'install-argv)
           (object/replace-field
             platform
             'install-argv
             (proc (object/ref platform 'install-argv)))
           platform)))
      (if (object/has-field? with-install-argv 'install-argvs)
        (object/replace-field
          with-install-argv
          'install-argvs
          (vector/map proc (object/ref with-install-argv 'install-argvs)))
        with-install-argv)))

  (doc-next
    (summary "Apply a transformation to all install argv fields in a package action.")
    (param
      'package-action
      "Package action that may contain `install-argv`, `install-argvs`, or platform overrides.")
    (param 'proc "Procedure that receives and returns an argv vector."))

  (define (package/map-install-argvs package-action proc)
    (let
      ((with-install-argv
         (if (object/has-field? package-action 'install-argv)
           (object/replace-field
             package-action
             'install-argv
             (proc (object/ref package-action 'install-argv)))
           package-action)))
      (let
        ((with-install-argvs
           (if (object/has-field? with-install-argv 'install-argvs)
             (object/replace-field
               with-install-argv
               'install-argvs
               (vector/map proc (object/ref with-install-argv 'install-argvs)))
             with-install-argv)))
        (if (object/has-field? with-install-argvs 'platforms)
          (object/replace-field
            with-install-argvs
            'platforms
            (vector/map
              (lambda (platform) (map-package-platform-install-argvs platform proc))
              (object/ref with-install-argvs 'platforms)))
          with-install-argvs))))

  (doc-next (summary "Transform package install argv fields inside a tool's action."))

  (define (tool/map-package-install-argvs tool-value proc)
    (object/replace-field
      tool-value
      'action
      (package/map-install-argvs (object/ref tool-value 'action) proc)))

  (doc-next (summary "Transform each check argv vector in a tool."))

  (define (tool/map-check-argvs tool-value proc)
    (if (object/has-field? tool-value 'checks)
      (object/replace-field
        tool-value
        'checks
        (vector/map
          (lambda (check-value)
            (object/replace-field
              check-value
              'argv
              (proc (object/ref check-value 'argv))))
          (object/ref tool-value 'checks)))
      tool-value))

  (doc-next
    (summary "Transform each uninstall command argv vector.")
    (param 'uninstall-value "Uninstall metadata object.")
    (param 'proc "Procedure that receives and returns an argv vector."))

  (define (uninstall/map-command-argvs uninstall-value proc)
    (if (object/has-field? uninstall-value 'commands)
      (object/replace-field
        uninstall-value
        'commands
        (vector/map
          (lambda (command-value)
            (object/replace-field
              command-value
              'argv
              (proc (object/ref command-value 'argv))))
          (object/ref uninstall-value 'commands)))
      uninstall-value))

  (doc-next (summary "Transform uninstall command argv fields inside a tool."))

  (define (tool/map-uninstall-command-argvs tool-value proc)
    (if (object/has-field? tool-value 'uninstall)
      (object/replace-field
        tool-value
        'uninstall
        (uninstall/map-command-argvs (object/ref tool-value 'uninstall) proc))
      tool-value))

  (doc-next
    (signature "(install-argv/prepend obj argv ...)")
    (summary "Prepend argv parts to an object's `install-argv` vector."))

  (define (install-argv/prepend obj . argv)
    (object/replace-field
      obj
      'install-argv
      (vector/append (list->vector argv) (object/ref obj 'install-argv (arr)))))

  (doc-next (summary "Append a Scheme list of argv items to an argv vector."))

  (define (argv/append prefix items) (arr/append-list prefix items))

  (moduledoc
    (summary "Catalog object and argv transformation helpers.")
    (group "Transformations")))
