(import (rnrs) (scaffold catalog) (acme tools))

(catalog (acme-tool "demo" "demo-pkg"))

(moduledoc (summary "Catalog fixture that consumes a local extension library."))
