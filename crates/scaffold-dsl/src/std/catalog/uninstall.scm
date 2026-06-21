(library
  (scaffold catalog uninstall)
  (export
    uninstall
    uninstall/command
    host/uninstall-command
    uninstall/path
    host/uninstall-path)
  (import (rnrs) (scaffold config))

  (doc-next
    (signature "(uninstall field ...)")
    (summary "Create uninstall metadata for a catalog tool.")
    (param
      'field
      "Fields such as `commands`, `paths`, `remove-bins`, or `remove-prefix`."))

  (define uninstall object)

  (doc-next
    (signature "(uninstall/command argv field ...)")
    (summary "Create a command run by `scaffold uninstall`.")
    (param 'argv "Command argv vector. Supports normal Scaffold template bindings."))

  (define (uninstall/command argv . fields) (cons* (field 'argv argv) fields))

  (doc-next
    (signature "(host/uninstall-command predicate argv field ...)")
    (summary "Create an uninstall command that only runs on matching hosts."))

  (define (host/uninstall-command predicate-value argv . fields)
    (cons* (field 'argv argv) (field 'when predicate-value) fields))

  (doc-next
    (signature "(uninstall/path path field ...)")
    (summary "Create a path removed by `scaffold uninstall`.")
    (param 'path "File or directory path. Supports normal Scaffold template bindings."))

  (define (uninstall/path path . fields) (cons* (field 'path path) fields))

  (doc-next
    (signature "(host/uninstall-path predicate path field ...)")
    (summary "Create an uninstall path that is only removed on matching hosts."))

  (define (host/uninstall-path predicate-value path . fields)
    (cons* (field 'path path) (field 'when predicate-value) fields))

  (moduledoc (summary "Catalog uninstall metadata constructors.") (group "Catalog")))
