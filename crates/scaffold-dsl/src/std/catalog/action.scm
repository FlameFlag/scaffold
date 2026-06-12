(library
  (scaffold catalog action)
  (export action required package build)
  (import (rnrs) (scaffold config))

  (doc-next
    (signature "(action ...)")
    (summary "Create an action object with a `type` field.")
    (hidden))

  (define (action type . fields) (apply object (field 'type type) fields))

  (doc-next
    (summary "Create an action for commands Scaffold should require but not install.")
    (returns "An action object with type `required`."))

  (define (required) (action 'required))

  (doc-next
    (signature "(package field ...)")
    (summary "Create a package install action.")
    (param
      'field
      "Package fields such as `name`, `install-argv`, `install-argvs`, or `platforms`.")
    (returns "An action object with type `package`."))

  (define (package . fields) (apply action 'package fields))

  (doc-next
    (signature "(build field ...)")
    (summary "Create a build action for tools compiled from source.")
    (param 'field "Build fields such as `path`, `argv`, or ordered `argvs`."))

  (define (build . fields) (apply action 'build fields))

  (moduledoc (summary "Catalog action constructors.") (group "Actions")))
