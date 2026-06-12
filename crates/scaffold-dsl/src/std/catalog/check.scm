(library
  (scaffold catalog check)
  (export check host/check)
  (import (rnrs) (scaffold config))

  (doc-next
    (signature "(check argv field ...)")
    (summary "Create a command check run to detect whether a tool is already present."))

  (define (check argv . fields) (apply object (field 'argv argv) fields))

  (doc-next
    (signature "(host/check predicate argv field ...)")
    (summary "Create a check that only applies when the host matches a predicate."))

  (define (host/check predicate-value argv . fields)
    (apply check argv (field 'when predicate-value) fields))

  (moduledoc (summary "Catalog presence check constructors.") (group "Checks")))
