(import (rnrs) (scaffold config))

(moduledoc
  (summary "Fixture for top-level helper definitions that construct output data."))

(define field cons)

(define object list)

(extern-doc field
  (signature "(field name value)")
  (summary "Local fixture helper for object fields."))

(extern-doc object
  (signature "(object field ...)")
  (summary "Local fixture helper for object values."))

(object (field 'name "demo"))
