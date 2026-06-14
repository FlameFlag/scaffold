(import (rnrs) (scaffold catalog))

(catalog
  (tool "demo"
    (archive/strip-components "archives/demo.tar.gz" 1)
    (field 'bins (arr (bin "demo")))))
