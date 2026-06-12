(library
  (scaffold extensions platform windows shell)
  (export
    powershell/tool
    pwsh/tool
    cmd/tool
    where/tool
    powershell/command-argv
    pwsh/command-argv
    cmd/c-argv
    where/argv)
  (import (rnrs) (scaffold catalog base) (scaffold extensions platform windows base))

  (doc-next
    (signature "(powershell/tool field ...)")
    (summary "Create a required descriptor for legacy Windows PowerShell `powershell`."))

  (define (powershell/tool . fields) (apply windows/command-tool "powershell" fields))

  (doc-next
    (signature "(pwsh/tool field ...)")
    (summary
      "Create a required descriptor for modern cross-platform PowerShell `pwsh`."))

  (define (pwsh/tool . fields) (apply windows/command-tool "pwsh" fields))

  (doc-next
    (signature "(cmd/tool field ...)")
    (summary "Create a required descriptor for `cmd`."))

  (define (cmd/tool . fields) (apply windows/command-tool "cmd" fields))

  (doc-next
    (signature "(where/tool field ...)")
    (summary "Create a required descriptor for command lookup tool `where`."))

  (define (where/tool . fields) (apply windows/command-tool "where" fields))

  (doc-next
    (summary
      "Build argv for running a legacy Windows PowerShell command without profile loading.")
    (param 'script "PowerShell command text.")
    (returns "Vector argv beginning with `powershell`."))

  (define (powershell/command-argv script)
    (arr "powershell" "-NoProfile" "-ExecutionPolicy" "Bypass" "-Command" script))

  (doc-next
    (summary
      "Build argv for running a modern PowerShell command without profile loading.")
    (param 'script "PowerShell command text.")
    (returns "Vector argv beginning with `pwsh`."))

  (define (pwsh/command-argv script) (arr "pwsh" "-NoProfile" "-Command" script))

  (doc-next
    (summary "Build argv for running a command through `cmd /C`.")
    (param 'command "Command text passed to `cmd /C`.")
    (returns "Vector argv beginning with `cmd`."))

  (define (cmd/c-argv command) (arr "cmd" "/C" command))

  (doc-next
    (summary "Build argv for locating a command on Windows PATH.")
    (param 'command "Command name to locate.")
    (returns "Vector argv for `where`."))

  (define (where/argv command) (arr "where" command))

  (moduledoc
    (summary "Windows shell and command lookup helpers.")
    (group "Windows tools")))
