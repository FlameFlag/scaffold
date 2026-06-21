(library
  (examples tools desktop)
  (export vscode desktop/tools)
  (import
    (rnrs)
    (scaffold catalog)
    (scaffold extensions app flatpak)
    (scaffold extensions app winget))

  (moduledoc (summary "Desktop application examples.") (group "Examples"))

  (doc-next (summary "VS Code from platform application installers."))

  (define vscode
    (tool
      "vscode"
      (package
        (field 'name "visual-studio-code")
        (field
          'platforms
          (arr
            (flatpak/package-platform "com.visualstudio.code")
            (winget/package-platform "Microsoft.VisualStudioCode"))))
      (field 'bins (arr (bin "code")))
      (field
        'checks
        (arr
          (host/check 'linux (arr "flatpak" "info" "com.visualstudio.code"))
          (host/check
            'windows
            (arr "winget" "list" "--id" "Microsoft.VisualStudioCode" "--exact"))))
      (meta
        (description "Editor installed through Flatpak on Linux or WinGet on Windows.")
        (home-page "https://code.visualstudio.com/")
        (license "Proprietary")
        (tags "editor" "desktop")
        (main-program "code"))))

  (doc-next (summary "Return desktop application examples."))

  (define (desktop/tools) (list vscode)))
