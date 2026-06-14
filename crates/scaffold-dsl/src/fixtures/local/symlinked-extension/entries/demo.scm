(library
  (entries demo)
  (export demo)
  (import (rnrs) (scaffold catalog))

  (define demo (tool "demo" (required))))
