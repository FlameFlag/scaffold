(library
  (scaffold extensions platform windows)
  (export
    windows/command-tool
    powershell/tool
    pwsh/tool
    cmd/tool
    reg/tool
    where/tool
    powershell/command-argv
    pwsh/command-argv
    cmd/c-argv
    reg/query-argv
    reg/add-argv
    where/argv)
  (import
    (rnrs)
    (scaffold catalog base)
    (scaffold extensions platform windows base)
    (scaffold extensions platform windows shell)
    (scaffold extensions platform windows registry))

  (moduledoc
    (summary
      "Windows command descriptors and argv helpers for shell, command lookup, and Registry setup tasks.")
    (group "Windows tools")))
