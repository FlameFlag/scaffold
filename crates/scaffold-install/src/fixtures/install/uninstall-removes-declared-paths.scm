(import (rnrs) (scaffold catalog))

(catalog
  (tool "demo"
    (required)
    (field 'uninstall
      (uninstall
        (field 'paths (arr (uninstall/path "{{ root_dir }}/trash")))))))
