(library
  (scaffold extensions platform macos base)
  (export macos/command-tool)
  (import (rnrs) (scaffold catalog base))

  (doc-next
    (signature "(macos/command-tool command field ...)")
    (summary "Create a macOS-only required command descriptor.")
    (param 'command "Command name expected on macOS.")
    (param 'field "Additional tool fields.")
    (returns "A required catalog tool constrained to macOS."))

  (define (macos/command-tool command . fields)
    (apply
      tool
      command
      (required)
      (field 'platforms (arr 'macos))
      (field 'bins (arr (bin command)))
      fields))

  (moduledoc (summary "Shared macOS command descriptor helpers.") (group "macOS tools")))
