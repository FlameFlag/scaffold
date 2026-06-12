(import (rnrs) (scaffold catalog) (scaffold test) (acme tools))

(define demo (wrapped-tool "demo"))

(assert/equal "demo" (object/ref demo 'name))

(assert/equal "wrapped-package" (object/ref (object/ref demo 'action) 'name))

(moduledoc
  (summary "Assertions for local extension libraries that depend on each other."))

(extern-doc demo
  (signature "(demo ...)")
  (summary "Wrapped local extension tool used by dependency assertions."))
