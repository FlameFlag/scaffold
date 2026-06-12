(import (rnrs) (scaffold config))

(moduledoc (summary "Fixture with valid Scheme syntax for diagnostics tests."))

(define x 1)

(extern-doc x (signature "x") (summary "Fixture value."))
