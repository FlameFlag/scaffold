(library
  (scaffold catalog root)
  (export catalog catalog/tool required/tool)
  (import (rnrs) (scaffold config))

  (doc-next
    (signature "(catalog tool ...)")
    (summary "Build the top-level catalog object from one or more tool objects.")
    (param
      'tool
      "Tool objects produced by `tool`, `required/tool`, or extension helpers.")
    (returns "An object with a `tools` vector ready for Scaffold catalog output."))

  (define (catalog . tools) (object (field 'tools (list->vector tools))))

  (doc-next
    (signature "(catalog/tool name action field ...)")
    (summary "Create a raw catalog tool list for macro-oriented helpers.")
    (param 'name "Displayed tool name.")
    (param 'action "Action object such as `package` or `required`.")
    (param 'field "Additional raw fields added to the tool.")
    (returns "A raw association-list representation of a catalog tool.")
    (markdown
      "Prefer `tool` for ordinary catalog entries. Use `catalog/tool` when writing extension macros that need to splice fields directly into the raw catalog shape before Scaffold normalizes it.")
    (example
      "(catalog/tool\n  \"rg\"\n  (package \"ripgrep\")\n  (field 'bins (arr (bin \"rg\"))))")
    (see 'tool))

  (define-syntax catalog/tool
    (syntax-rules
      ()
      ((_ name action-value field ...)
        (list (cons "name" name) (cons "action" action-value) field ...))))

  (doc-next
    (signature "(required/tool name field ...)")
    (summary "Create a raw catalog tool that must already exist on the host.")
    (param 'name "Required command name.")
    (param 'field "Additional raw fields added to the tool.")
    (returns "A raw tool list with a `required` action.")
    (markdown
      "This is the raw macro form behind higher-level required command helpers. Prefer `tool` with `required`, or an extension helper, unless a macro needs to splice raw fields.")
    (example "(required/tool \"git\" (field 'bins (arr (bin \"git\"))))")
    (see 'tool)
    (see 'required))

  (define-syntax required/tool
    (syntax-rules
      ()
      ((_ name field ...)
        (list
          (cons "name" name)
          (cons "action" (list (cons "type" "required")))
          field
          ...))))

  (moduledoc (summary "Core catalog object constructors.") (group "Catalog")))
