(import (rnrs) (scaffold catalog))

(catalog
  (tool "demo"
    (dmg "archives/demo.dmg")
    (field 'bins (arr (bin "demo")))))
