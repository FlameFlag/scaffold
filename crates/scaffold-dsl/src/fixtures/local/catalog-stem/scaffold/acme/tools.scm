(library
  (acme tools)
  (export demo-tool)
  (import (rnrs) (scaffold catalog))

  (define (demo-tool) (tool "demo" (required))))
