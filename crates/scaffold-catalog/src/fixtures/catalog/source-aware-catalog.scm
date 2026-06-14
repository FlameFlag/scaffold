(import (rnrs) (scaffold catalog))

(tool "ok" (required))

(tool "bad" (required) (field 'surprise #t))
