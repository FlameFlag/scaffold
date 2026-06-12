(import (rnrs) (scaffold config))

(moduledoc (summary "Fixture for extracting LSP documentation entries."))

(extern-doc demo
  (signature "(demo name)")
  (summary "Create a demo.")
  (markdown "More docs.")
  (param 'name "Name for the demo.")
  (returns "A demo object.")
  (group "Fixtures")
  (see 'other-demo)
  (example "(demo \"x\")"))
