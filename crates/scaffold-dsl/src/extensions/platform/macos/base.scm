(library
  (scaffold extensions platform macos base)
  (export
    macos/command-tool
    macos/xcode-command-line-tools-platform
    macos/zip-app-bin-platform)
  (import (rnrs) (scaffold catalog base) (scaffold extensions support download))

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

  (doc-next
    (summary "Create a macOS installer that delegates Git to Apple Command Line Tools."))

  (define (macos/xcode-command-line-tools-platform)
    (package/platform
      'macos
      (arr "sh" "xcode-select")
      (arr "sh" "-c" "xcode-select -p >/dev/null 2>&1 || xcode-select --install")))

  (doc-next
    (summary "Create a macOS installer for an app zip with a CLI shim inside it.")
    (param 'predicate-value "Host predicate for this package rule.")
    (param 'tool-name "Tool cache directory name.")
    (param 'url "Archive URL. This may be a stable latest endpoint.")
    (param 'archive-name "Archive filename used in the tool cache.")
    (param 'app-name "App bundle name inside the archive.")
    (param 'bin-relative-path "CLI path relative to the app bundle.")
    (param 'bin-name "Installed executable name."))

  (define
    (macos/zip-app-bin-platform
      predicate-value
      tool-name
      url
      archive-name
      app-name
      bin-relative-path
      bin-name)
    (let*
      ((root (tool-cache-dir tool-name))
        (archive (downloaded-archive-path tool-name archive-name))
        (extract-dir (archive-extract-dir tool-name))
        (app-path (string-append root "/" app-name)))
      (package/platform-argvs
        predicate-value
        (arr "curl" "ditto" "ln" "mkdir" "rm")
        (arr
          (arr "mkdir" "-p" root "{{ bin_dir }}")
          (arr "curl" "-fsSL" "--retry" "3" "-o" archive url)
          (arr "rm" "-rf" extract-dir app-path)
          (arr "mkdir" "-p" extract-dir)
          (arr "ditto" "-x" "-k" archive extract-dir)
          (arr "ditto" (string-append extract-dir "/" app-name) app-path)
          (arr
            "ln"
            "-sfn"
            (string-append app-path "/" bin-relative-path)
            (string-append "{{ bin_dir }}/" bin-name))))))

  (moduledoc
    (summary "Shared macOS command and installer helpers.")
    (group "macOS tools")))
