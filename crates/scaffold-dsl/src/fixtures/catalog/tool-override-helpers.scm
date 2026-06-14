(import (rnrs) (scaffold catalog))

(define base-tool
  (tool
    "demo"
    (required)
    (field 'bins (arr (bin "demo")))))

(catalog
  (tool/override
    base-tool
    (lambda (old)
      (object
        (field 'name (string-append (object/ref old 'name) "-nightly"))
        (field
          'bins
          (arr (bin (string-append (object/ref old 'name) "-nightly"))))))))
