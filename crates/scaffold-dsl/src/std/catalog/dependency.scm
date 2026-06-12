(library
  (scaffold catalog dependency)
  (export install/order depends install/before install/after)
  (import (rnrs) (scaffold config))

  (doc-next
    (summary
      "Set a numeric ordering priority for a tool within the resolved install graph.")
    (param
      'value
      "Lower values are installed earlier when dependency constraints allow."))

  (define (install/order value) (field 'order value))

  (doc-next
    (signature "(depends name ...)")
    (summary "Declare tools that must be installed before this tool.")
    (param
      'name
      "Tool name dependency. Requested installs include dependencies automatically."))

  (define (depends . names) (field 'depends (list->vector names)))

  (doc-next
    (signature "(install/before name ...)")
    (summary "Declare tools that should be installed after this tool."))

  (define (install/before . names) (field 'before (list->vector names)))

  (doc-next
    (signature "(install/after name ...)")
    (summary "Declare tools that should be installed before this tool."))

  (define (install/after . names) (field 'after (list->vector names)))

  (moduledoc
    (summary "Catalog install ordering and dependency field helpers.")
    (group "Catalog")))
