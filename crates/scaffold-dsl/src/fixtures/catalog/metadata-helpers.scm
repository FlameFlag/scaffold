(import (rnrs) (scaffold catalog))

(catalog
  (tool "demo"
    (required)
    (meta
      (home-page "https://example.test/demo")
      (description "Demo tool.")
      (license "MIT")
      (maintainers "flame")
      (tags "cli" "demo")
      (main-program "demo")
      (source "https://example.test/demo.git"))
    (passthru (field 'updater "manual"))))
