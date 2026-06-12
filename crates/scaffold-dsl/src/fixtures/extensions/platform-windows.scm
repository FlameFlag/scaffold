(import
  (rnrs)
  (scaffold catalog)
  (scaffold test)
  (scaffold extensions platform windows))

(assert/equal
  (arr
    "powershell"
    "-NoProfile"
    "-ExecutionPolicy"
    "Bypass"
    "-Command"
    "Get-Command rg")
  (powershell/command-argv "Get-Command rg"))

(assert/equal
  (arr "pwsh" "-NoProfile" "-Command" "Get-Command rg")
  (pwsh/command-argv "Get-Command rg"))

(assert/equal
  (arr "reg" "query" "HKCU\\Software\\Scaffold" "/v" "InstallDir")
  (reg/query-argv "HKCU\\Software\\Scaffold" "InstallDir"))

(assert/equal
  (arr
    "reg"
    "add"
    "HKCU\\Software\\Scaffold"
    "/v"
    "InstallDir"
    "/t"
    "REG_SZ"
    "/d"
    "C:\\Tools\\Demo"
    "/f")
  (reg/add-argv "HKCU\\Software\\Scaffold" "InstallDir" "REG_SZ" "C:\\Tools\\Demo"))

(assert/equal (arr "cmd" "/C" "where rg") (cmd/c-argv "where rg"))

(assert/equal (arr "where" "rg") (where/argv "rg"))

(doc-next
  (signature "(powershell-required ...)")
  (summary "Required PowerShell descriptor used by Windows extension assertions."))

(define powershell-required (powershell/tool))

(assert/equal "powershell" (object/ref powershell-required 'name))

(assert/equal 'windows (vector-ref (object/ref powershell-required 'platforms) 0))

(moduledoc (summary "Fixture for Windows platform helpers."))
