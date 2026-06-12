(library
  (scaffold config object)
  (export
    field
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
    object/append-field-vector
    object/append-vector)
  (import (rnrs) (scaffold core object) (scaffold core doc))

  (moduledoc (summary "Focused facade for Scaffold object helpers.") (group "Objects"))

  (extern-doc field-key=?
    (hidden)
    (summary "Compare two object field keys for equality."))

  (extern-doc field-name-member?
    (hidden)
    (summary "Return whether a field name appears in a list of names."))

  (extern-doc remove-field-names
    (hidden)
    (summary "Remove fields whose names appear in a name list."))

  (extern-doc merge-one
    (hidden)
    (summary "Merge one override object into a base object."))

  (extern-doc field
    (signature "(field name value)")
    (summary "Create one object field pair.")
    (param 'name "Symbol or string key to store in the object.")
    (param 'value "Any Scheme value that should become the field value.")
    (returns "A pair consumed by `object`, merge helpers, and catalog constructors."))

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

  (extern-doc object/append-field-vector
    (signature "(object/append-field-vector obj name values)")
    (summary "Append vector values to an object field."))

  (extern-doc object/append-vector
    (signature "(object/append-vector obj name value ...)")
    (summary "Append values to an object vector field.")))
