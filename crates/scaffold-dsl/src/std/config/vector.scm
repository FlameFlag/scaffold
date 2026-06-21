(library
  (scaffold config vector)
  (export arr list->arr vector/map vector/append arr/append-list arr/prepend-list)
  (import (rnrs) (scaffold core vector) (scaffold core doc))

  (moduledoc (summary "Focused facade for Scaffold vector helpers.") (group "Vectors"))

  (extern-doc append-vector-lists
    (hidden)
    (summary "Flatten a list of vectors into a Scheme list."))

  (extern-doc arr
    (signature "(arr value ...)")
    (summary "Create a Scaffold vector value.")
    (param 'value "Values to store in order.")
    (returns "A Scheme vector that serializes as a JSON array."))

  (extern-doc list->arr
    (signature "(list->arr values)")
    (summary "Convert a Scheme list into a Scaffold vector value."))

  (extern-doc vector/map
    (signature "(vector/map proc values)")
    (summary "Map a procedure over a vector and return a vector."))

  (extern-doc vector/append
    (signature "(vector/append vector ...)")
    (summary "Append any number of vectors into one vector."))

  (extern-doc arr/append-list
    (signature "(arr/append-list values suffix)")
    (summary "Append a Scheme list to a Scaffold vector."))

  (extern-doc arr/prepend-list
    (signature "(arr/prepend-list prefix values)")
    (summary "Prepend a Scheme list to a Scaffold vector.")))
