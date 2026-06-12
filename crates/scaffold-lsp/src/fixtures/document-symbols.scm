(import (rnrs) (scaffold catalog))

(moduledoc (summary "Fixture for document word and head-symbol extraction."))

(define café 1)

(extern-doc café
  (signature "café")
  (summary "Fixture value with a non-ASCII identifier."))

(tool "demo" (required))
