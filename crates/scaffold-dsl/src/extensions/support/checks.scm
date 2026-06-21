(library
  (scaffold extensions support checks)
  (export command/check)
  (import (rnrs) (scaffold catalog base))

  (doc-next
    (signature "(command/check argv ...)")
    (summary "Create a command check from plain argv parts.")
    (param 'argv "Command and arguments Scaffold should run to verify presence.")
    (returns "A `check` object with `argv` converted to a vector.")
    (example "(command/check \"git\" \"--version\")"))

  (define (command/check . argv) (check (list->arr argv)))

  (moduledoc (summary "Convenience constructors for catalog checks.") (group "Checks")))
