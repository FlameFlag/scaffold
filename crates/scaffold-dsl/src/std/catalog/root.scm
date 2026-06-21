(library
  (scaffold catalog root)
  (export catalog catalog/from-lists tool-list/from-lists catalog/tool required/tool)
  (import (rnrs) (scaffold config))

  (doc-next
    (signature "(catalog tool ...)")
    (summary "Build the top-level catalog object from one or more tool objects.")
    (param
      'tool
      "Tool objects produced by `tool`, `required/tool`, or extension helpers.")
    (returns "An object with a `tools` vector ready for Scaffold catalog output."))

  (define (catalog . tools) (object (field 'tools (list->arr tools))))

  (doc-next
    (signature "(catalog/from-lists tool-list ...)")
    (summary "Build a catalog by appending multiple Scheme lists of tool objects.")
    (param
      'tool-list
      "A Scheme list of tool objects, such as the result of a grouped `*/tools` helper.")
    (returns "An object with a `tools` vector containing every tool in order.")
    (markdown
      "Use this when catalogs are organized into modules that each return a list of tools. It avoids the usual `(apply catalog (append ...))` boilerplate while preserving list order.")
    (example
      "(catalog/from-lists\n  (prerequisites/tools)\n  (ecosystem/tools)\n  (apps/tools))")
    (see 'catalog))

  (define (catalog/from-lists . tool-lists)
    (object (field 'tools (list->arr (fold-right append '() tool-lists)))))

  (doc-next
    (signature "(tool-list/from-lists tool-list ...)")
    (summary "Append grouped Scheme lists of tool objects into one tool list.")
    (param
      'tool-list
      "A Scheme list of tool objects, such as the result of a grouped `*/tools` helper.")
    (returns "A Scheme list containing every tool in order.")
    (markdown
      "Use this inside catalog library modules that return lists of tools. It is the list-returning companion to `catalog/from-lists`.")
    (example
      "(define (desktop/tools)\n  (tool-list/from-lists\n    (terminal/tools)\n    (editor/tools)))")
    (see 'catalog/from-lists))

  (define (tool-list/from-lists . tool-lists) (fold-right append '() tool-lists))

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
