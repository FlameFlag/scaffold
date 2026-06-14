(import
  (rnrs)
  (only (scaffold catalog root) catalog)
  (only (scaffold catalog action) required)
  (only (scaffold catalog tool) tool bin override)
  (only (scaffold config vector) arr)
  (only (scaffold config object) object field object/ref))

(define base
  (tool "demo" (required) (field 'bins (arr (bin "demo")))))

(catalog
  (override
    base
    (lambda (old)
      (object
        (field 'name (string-append (object/ref old 'name) "-scoped"))
        (field 'bins (arr (bin (string-append (object/ref old 'name) "-scoped"))))))))
