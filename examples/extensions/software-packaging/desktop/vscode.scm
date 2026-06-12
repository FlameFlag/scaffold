(library
  (software-packaging desktop vscode)
  (export vscode)
  (import
    (rnrs)
    (scaffold catalog)
    (scaffold extensions app flatpak)
    (scaffold extensions app winget))

  (doc-next (summary "Example Visual Studio Code desktop tool."))

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
        (description "VS Code installed from platform package/application installers.")
        (home-page "https://code.visualstudio.com/")
        (tags "editor")))))
