(library
  (examples tools packages)
  (export ripgrep package/tools)
  (import
    (rnrs)
    (scaffold catalog)
    (scaffold extensions app winget)
    (scaffold extensions distro apt)
    (scaffold extensions distro dnf)
    (scaffold extensions distro pacman))

  (moduledoc (summary "Cross-platform package manager examples.") (group "Examples"))

  (doc-next
    (summary "ripgrep package with Linux distro and Windows package platforms."))

  (define ripgrep
    (tool
      "ripgrep"
      (package
        (field 'name "ripgrep")
        (field
          'platforms
          (arr
            (apt/package-platform "ripgrep")
            (dnf/package-platform "ripgrep")
            (pacman/package-platform "ripgrep")
            (winget/package-platform "BurntSushi.ripgrep.MSVC"))))
      (field 'bins (arr (bin/version "rg" "--version")))
      (field
        'checks
        (arr
          (host/check 'linux (arr "rg" "--version"))
          (host/check 'windows (arr "rg" "--version"))))
      (meta
        (description
          "Fast recursive search installed through the host package manager.")
        (home-page "https://github.com/BurntSushi/ripgrep")
        (license "Unlicense OR MIT")
        (tags "search" "cli")
        (main-program "rg"))))

  (doc-next (summary "Return package-manager backed example tools."))

  (define (package/tools) (list ripgrep)))
