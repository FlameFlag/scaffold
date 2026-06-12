(library
  (acme tools)
  (export wrapped-tool)
  (import (rnrs) (scaffold config) (acme base))

  (doc 'wrapped-tool
    (signature "(wrapped-tool ...)")
    (summary "Wrapper helper used to verify local extension dependency ordering."))

  (define (wrapped-tool name) (base-tool name "wrapped-package"))

  (moduledoc
    (summary "Fixture extension library that wraps helpers from the Acme base library.")))
