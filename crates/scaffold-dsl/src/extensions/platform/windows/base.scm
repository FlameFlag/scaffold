(library
  (scaffold extensions platform windows base)
  (export windows/command-tool)
  (import (rnrs) (scaffold catalog base))

  (doc-next
    (signature "(windows/command-tool command field ...)")
    (summary "Create a Windows-only required command descriptor.")
    (param 'command "Command name expected on Windows.")
    (param 'field "Additional tool fields.")
    (returns "A required catalog tool constrained to Windows."))

  (define (windows/command-tool command . fields)
    (apply
      tool
      command
      (required)
      (field 'platforms (arr 'windows))
      (field 'bins (arr (bin command)))
      fields))

  (moduledoc
    (summary "Shared Windows command descriptor helpers.")
    (group "Windows tools")))
