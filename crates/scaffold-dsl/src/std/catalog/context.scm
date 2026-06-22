(library
  (scaffold catalog context)
  (export catalog/mode catalog/mode?)
  (import (rnrs) (scaffold config))

  (extern-doc catalog/mode
    (signature "catalog/mode")
    (summary "Mode selected for the current catalog evaluation.")
    (returns "A mode string supplied by `--catalog-mode`, or `#f` when absent."))

  (define (injected-string present value)
    (and (string=? present "#t") value))

  (define catalog/mode
    (injected-string "{{ catalog_mode_present }}" "{{ catalog_mode }}"))

  (doc-next
    (signature "(catalog/mode? value)")
    (summary "Return whether `catalog/mode` matches a mode string.")
    (param 'value "Mode string to compare with the active catalog mode."))

  (define (catalog/mode? value)
    (and catalog/mode (string=? catalog/mode value)))

  (moduledoc
    (summary "Read-only catalog evaluation facts injected by Scaffold.")
    (group "Catalog")
    (effect 'context-read-only)
    (requires-capability 'scaffold.catalog-context)))
