(import (rnrs) (scaffold catalog) (scaffold test) (acme tools))

(define demo (acme-tool "nested" "nested-pkg"))

(assert/equal "nested" (object/ref demo 'name))

(assert/equal "nested-pkg" (object/ref (object/ref demo 'action) 'name))

(moduledoc (summary "Assertions for the local Acme extension helper."))

(extern-doc demo
  (signature "(demo ...)")
  (summary "Demo tool produced by the Acme extension helper."))
