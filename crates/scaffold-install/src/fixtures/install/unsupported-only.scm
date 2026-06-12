(import (rnrs) (scaffold catalog))

(catalog
  (tool "unsupported"
    (required)
    (field 'platforms (arr '{{ unsupported_host_os }}))))
