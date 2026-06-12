(library
  (scaffold extensions platform macos)
  (export macos/command-tool)
  (import (rnrs) (scaffold catalog base) (scaffold extensions platform macos base))

  (moduledoc
    (summary "macOS platform helpers for declaring required host commands.")
    (group "macOS tools")))
