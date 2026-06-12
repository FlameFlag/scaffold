(import (rnrs) (scaffold config))

(moduledoc (summary "Fixture for emitting multiple top-level output values."))

(define field cons)

(define object list)

(extern-doc field
  (signature "(field name value)")
  (summary "Local fixture helper for object fields."))

(extern-doc object
  (signature "(object field ...)")
  (summary "Local fixture helper for object values."))

(object (field 'name "one"))

(object (field 'name "two"))
