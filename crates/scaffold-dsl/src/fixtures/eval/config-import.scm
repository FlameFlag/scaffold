(import (rnrs) (scaffold config))

(object (field 'name "demo") (field 'argv (arr "cargo" "test")))

(moduledoc (summary "Fixture library used to verify shared Scaffold config imports."))
