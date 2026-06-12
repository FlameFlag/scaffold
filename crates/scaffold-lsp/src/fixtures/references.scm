(import (rnrs) (scaffold catalog))

(moduledoc (summary "Fixture for symbol reference scanning."))

(doc-next
  (summary "Return `value` for reference scanning tests.")
  (param 'value "Value to return."))

(define (local-helper value) value)

(local-helper "ok")

(doc-next (summary "Fixture value that calls `local-helper`."))

(define local-value (local-helper "again"))

"local-helper in a string"

; local-helper in a comment
