(import (rnrs) (scaffold config))

(moduledoc (summary "Fixture for reading basic Scheme data into Scaffold JSON values."))

(let ((field cons) (object list) (arr vector))
  (object
    (field 'name "demo")
    (field 'argv (arr "cargo" "test"))
    (field 'nested-value (object (field 'kind 'package)))))
