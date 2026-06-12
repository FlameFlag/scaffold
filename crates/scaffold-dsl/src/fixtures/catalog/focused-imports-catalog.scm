(import (rnrs) (scaffold catalog base) (scaffold extensions ecosystem uv))

(let
  ((package-action
     (package
       (field 'name "demo")
       (field 'install-argv (arr "install" "{{ package }}")))))
  (catalog
    (tool "package-demo" (install-argv/prepend package-action "sudo"))
    (uv/tool "ruff")))

(moduledoc
  (summary
    "Fixture for importing focused catalog modules instead of the whole catalog API."))
