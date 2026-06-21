(library
  (scaffold catalog tool)
  (export tool inherit override tool/append-bins bin bin/version)
  (import (rnrs) (scaffold config))

  (doc-next
    (signature "(tool name action field ...)")
    (summary "Create a catalog tool object.")
    (param 'name "Tool name used in catalog output.")
    (param 'action "Action object describing how the tool is installed or required.")
    (param 'field "Additional fields such as `bins`, `checks`, or `platforms`.")
    (returns "A tool object accepted by `catalog`.")
    (see 'package)
    (see 'required))

  (define (tool name action-value . fields)
    (cons* (field 'name name) (field 'action action-value) fields))

  (doc-next
    (signature "(inherit base field ...)")
    (summary "Derive a tool object by replacing fields on an existing tool."))

  (define inherit object/inherit)

  (doc-next
    (summary "Derive a tool from overrides computed from the existing tool object.")
    (param 'base "Tool object to derive from.")
    (param
      'proc
      "Procedure that receives `base` and returns an object of replacement fields."))

  (define (override base proc) (object/override base proc))

  (doc-next
    (signature "(tool/append-bins obj bin ...)")
    (summary "Append binary descriptors to an object's `bins` vector."))

  (define (tool/append-bins obj . bins)
    (object/append-field-vector obj 'bins (list->arr bins)))

  (doc-next
    (signature "(bin name field ...)")
    (summary "Describe a binary exposed by an installed tool."))

  (define (bin name . fields) (cons* (field 'name name) fields))

  (doc-next
    (signature "(bin/version name argv ...)")
    (summary "Describe a binary whose version can be queried by running the binary."))

  (define (bin/version name . argv)
    (bin name (field 'version-argv (arr/append-list (arr name) argv))))

  (moduledoc (summary "Catalog tool and binary constructors.") (group "Catalog")))
