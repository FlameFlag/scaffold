(import (rnrs) (scaffold catalog))

(catalog
  (tool "supported"
    (required)
    (field 'bins (arr (bin "{{ current_exe }}"))))
  (tool "unsupported"
    (required)
    (field 'platforms (arr '{{ unsupported_host_os }}))))
