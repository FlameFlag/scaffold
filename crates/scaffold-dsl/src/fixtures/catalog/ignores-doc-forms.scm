(import (rnrs) (scaffold catalog))

(extern-doc tool
  (signature "(tool name action field ...)")
  (summary "Define a catalog tool."))

(tool "demo" (required))
