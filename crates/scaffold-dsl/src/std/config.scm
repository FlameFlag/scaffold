(library
  (scaffold config)
  (export
    field
    field?
    field/name
    field/value
    object
    arr
    list->arr
    vector/map
    vector/append
    arr/append-list
    arr/prepend-list
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
    call-with-split-fields
    doc
    doc-next
    extern-doc
    moduledoc
    typedoc
    signature
    summary
    markdown
    example
    param
    returns
    group
    see
    effect
    requires-capability
    stability
    since
    deprecated
    hidden)
  (import
    (rnrs)
    (scaffold config vector)
    (scaffold config object)
    (scaffold config documentation))

  (moduledoc
    (summary
      "Public facade for Scaffold object, vector, transformation, and documentation helpers.")))
