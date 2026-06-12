(library
  (acme tools)
  (export acme-tool acme-helper)

  (doc-next
    (signature "(acme-tool name [mode])")
    (summary "Create an Acme tool fixture.")
    (param 'name "Tool name."))

  (define (acme-tool name) name)

  (doc-next (summary "Return the Acme helper fixture value."))

  (define (acme-helper value) value))
