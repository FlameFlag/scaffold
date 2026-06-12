(library
  (scaffold config documentation)
  (export
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
  (import (rnrs) (scaffold core doc))

  (moduledoc
    (summary "Focused facade for Scaffold documentation helpers.")
    (group "Documentation"))

  (extern-doc doc-field
    (hidden)
    (summary "Create one structured documentation metadata field."))

  (extern-doc signature
    (signature "(signature text)")
    (summary
      "Declare the primary callable shape shown in completion and signature help."))

  (extern-doc summary
    (signature "(summary text)")
    (summary
      "Provide the one-line summary used by completion lists and hover previews."))

  (extern-doc markdown
    (signature "(markdown text)")
    (summary "Add longer Markdown documentation for hover and generated references."))

  (extern-doc example
    (signature "(example text)")
    (summary "Attach a Scheme example that tooling renders as a code block."))

  (extern-doc param
    (signature "(param name text)")
    (summary "Document one parameter for signature help and generated references."))

  (extern-doc returns
    (signature "(returns text)")
    (summary "Document the result produced by a value or helper."))

  (extern-doc group
    (signature "(group text)")
    (summary "Place a documented symbol into a tooling group."))

  (extern-doc see
    (signature "(see subject)")
    (summary "Link a doc entry to a related symbol."))

  (extern-doc effect
    (signature "(effect name)")
    (summary "Record the effect category for a module or documented binding."))

  (extern-doc requires-capability
    (signature "(requires-capability name)")
    (summary "Record a host capability required by a module or documented binding."))

  (extern-doc stability
    (signature "(stability text)")
    (summary "Record stability guidance such as experimental or stable."))

  (extern-doc since
    (signature "(since text)")
    (summary "Record the Scaffold version that introduced the documented symbol."))

  (extern-doc deprecated
    (signature "(deprecated text)")
    (summary "Record replacement guidance and mark completions as deprecated."))

  (extern-doc hidden
    (signature "(hidden)")
    (summary "Hide an internal doc entry from public completion lists."))

  (extern-doc doc
    (signature "(doc subject field ...)")
    (summary "Attach parseable documentation metadata to a value, macro, or DSL form.")
    (param 'subject "Quoted symbol or string name of the documented entity.")
    (param
      'field
      "Structured metadata created by helpers such as `summary`, `param`, `returns`, and `example`.")
    (returns
      "A documentation object ignored as catalog output and indexed by editor tooling.")
    (see 'moduledoc)
    (see 'typedoc))

  (extern-doc doc-next
    (signature "(doc-next field ...)")
    (summary "Attach documentation metadata to the following definition.")
    (param
      'field
      "Documentation metadata fields inherited by the following definition.")
    (returns "A documentation object indexed as if it documented the next binding.")
    (see 'doc)
    (see 'extern-doc))

  (extern-doc extern-doc
    (signature "(extern-doc subject field ...)")
    (summary "Attach documentation metadata to a host-backed or generated binding.")
    (param 'subject "Symbol or string name of the documented external binding.")
    (param 'field "Documentation metadata fields.")
    (returns "A documentation object normalized to a regular doc entry.")
    (see 'doc-next))

  (extern-doc moduledoc
    (signature "(moduledoc field ...)")
    (summary "Attach parseable documentation metadata to a Scheme file or library.")
    (param 'field "Module-level metadata fields.")
    (returns "A documentation object for file and library tooling."))

  (extern-doc typedoc
    (signature "(typedoc subject field ...)")
    (summary "Attach parseable documentation metadata to a type-like DSL concept.")))
