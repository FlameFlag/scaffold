(import (rnrs) (scaffold catalog) (scaffold test) (acme tools))

(define demo (acme-tool "demo" "demo-pkg"))

(assert/equal "demo" (object/ref demo 'name))

(assert/equal "demo-pkg" (object/ref (object/ref demo 'action) 'name))

(assert/equal
  (arr "custom-install" "{{ package }}")
  (object/ref (object/ref demo 'action) 'install-argv))

(assert/true (object/has-field? demo 'has-cargo))

(moduledoc (summary "Assertions for loading a local extension library."))

(extern-doc demo
  (signature "(demo ...)")
  (summary "Demo tool imported from the local Acme extension."))
