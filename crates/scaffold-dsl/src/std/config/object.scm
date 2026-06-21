(library
  (scaffold config object)
  (export
    field
    field?
    field/name
    field/value
    object
    object/has-field?
    object/ref
    object/remove-fields
    object/replace-fields
    object/merge
    object/inherit
    object/override
    object/replace-field
    object/update-field
    object/map-vector-field
    object/append-field-vector
    object/append-vector
    split-fields
    call-with-split-fields)
  (import (rnrs) (scaffold core object) (scaffold core doc))

  (moduledoc (summary "Focused facade for Scaffold object helpers.") (group "Objects"))

  (extern-doc field-name-member?
    (hidden)
    (summary "Return whether a field name appears in a list of names."))

  (extern-doc remove-field-names
    (hidden)
    (summary "Remove fields whose names appear in a name list."))

  (extern-doc object/ref-default
    (hidden)
    (summary "Read a field value with an already resolved default."))

  (extern-doc merge-one
    (hidden)
    (summary "Merge one override object into a base object."))

  (extern-doc field
    (signature "(field name value)")
    (summary "Create one object field pair.")
    (param 'name "Symbol or string key to store in the object.")
    (param 'value "Any Scheme value that should become the field value.")
    (returns "A pair consumed by `object`, merge helpers, and catalog constructors."))

  (extern-doc field?
    (signature "(field? value)")
    (summary "Return whether a value has Scaffold field-pair shape."))

  (extern-doc field/name
    (signature "(field/name field)")
    (summary "Return the name part of a field pair."))

  (extern-doc field/value
    (signature "(field/value field)")
    (summary "Return the value part of a field pair."))

  (extern-doc object
    (signature "(object field ...)")
    (summary "Create a Scaffold object from field pairs.")
    (param 'field "A `(field name value)` pair.")
    (returns "A Scheme list of pairs that serializes as a JSON object."))

  (extern-doc object/has-field?
    (signature "(object/has-field? obj name)")
    (summary "Return whether an object contains a field."))

  (extern-doc object/ref
    (signature "(object/ref obj name [default])")
    (summary "Read a field value from an object with an optional default.")
    (param 'obj "Object created with `object` or a catalog helper.")
    (param 'name "Field name to find.")
    (param 'default "Optional value returned when the field is absent.")
    (returns
      "The field value, the optional default, or `#f` when no default is supplied."))

  (extern-doc object/remove-fields
    (signature "(object/remove-fields obj name ...)")
    (summary "Return an object with named fields removed."))

  (extern-doc object/replace-fields
    (signature "(object/replace-fields obj field ...)")
    (summary "Return an object with fields replaced by name."))

  (extern-doc object/merge
    (signature "(object/merge base override ...)")
    (summary "Merge objects with later fields replacing earlier fields."))

  (extern-doc object/inherit
    (signature "(object/inherit base field ...)")
    (summary "Create a derived object by replacing fields on a base object."))

  (extern-doc object/override
    (signature "(object/override base proc)")
    (summary "Create a derived object from overrides computed from the base object.")
    (param 'base "Object to derive from.")
    (param
      'proc
      "Procedure that receives `base` and returns an object of replacement fields."))

  (extern-doc object/replace-field
    (signature "(object/replace-field obj name value)")
    (summary "Replace one named field on an object."))

  (extern-doc object/update-field
    (signature "(object/update-field obj name proc)")
    (summary "Transform an existing object field, leaving the object unchanged when the field is absent.")
    (param 'obj "Object created with `object` or a catalog helper.")
    (param 'name "Field name to transform.")
    (param 'proc "Procedure called with the current field value."))

  (extern-doc object/map-vector-field
    (signature "(object/map-vector-field obj name proc)")
    (summary "Map a procedure over an existing vector field, leaving the object unchanged when the field is absent.")
    (param 'obj "Object created with `object` or a catalog helper.")
    (param 'name "Vector field name to transform.")
    (param 'proc "Procedure mapped over each vector item."))

  (extern-doc object/append-field-vector
    (signature "(object/append-field-vector obj name values)")
    (summary "Append vector values to an object field."))

  (extern-doc object/append-vector
    (signature "(object/append-vector obj name value ...)")
    (summary "Append values to an object vector field."))

  (extern-doc split-fields
    (signature "(split-fields values)")
    (summary "Split mixed option values into non-field values and object fields.")
    (param 'values "A Scheme list containing ordinary option values and `(field name value)` pairs.")
    (returns "A pair whose car is non-field values and whose cdr is field values, preserving order."))

  (extern-doc call-with-split-fields
    (signature "(call-with-split-fields values proc)")
    (summary "Call a procedure with non-field values and object fields split apart.")
    (param 'values "A Scheme list containing ordinary option values and `(field name value)` pairs.")
    (param 'proc "Procedure called as `(proc ordinary-values fields)`.")))
