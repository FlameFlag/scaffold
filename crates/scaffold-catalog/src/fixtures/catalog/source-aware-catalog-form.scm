(import (rnrs) (scaffold catalog))

(catalog
  (tool "ok" (required))
  (tool "bad" (required) (field 'surprise #t)))
