(import (rnrs) (scaffold catalog))

(moduledoc (summary "Fixture for doc-driven parameter inlay hints."))

(doc-next (summary "Tool fixture used by inlay hint tests."))

(define demo-tool
  (tool "demo" (package (field 'name "demo")) (field 'bins (arr (bin "demo")))))
