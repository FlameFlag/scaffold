(import (rnrs) (scaffold catalog))

(catalog
  (tool "supported"
    (required)
    (field 'bins (arr (bin "{{ current_exe }}"))))
  (tool "platform-unsupported"
    (package
      (field
        'platforms
        (arr
          (package/platform
            '{{ current_host_os }}
            (arr "definitely-not-a-real-scaffold-installer")
            (arr "definitely-not-a-real-scaffold-installer" "install")))))))
