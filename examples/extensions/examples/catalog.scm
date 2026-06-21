(library
  (examples catalog)
  (export examples/catalog)
  (import
    (rnrs)
    (scaffold catalog)
    (examples targets distrobox)
    (examples tools builds)
    (examples tools desktop)
    (examples tools ecosystem)
    (examples tools packages)
    (examples tools prerequisites))

  (moduledoc (summary "Composed catalog for the Scaffold examples.") (group "Examples"))

  (doc-next (summary "Build the complete example catalog from focused tool groups."))

  (define (examples/catalog)
    (apply
      catalog
      (append
        (prerequisite/tools)
        (append
          (package/tools)
          (append
            (desktop/tools)
            (append (ecosystem/tools) (append (distrobox/tools) (build/tools)))))))))
