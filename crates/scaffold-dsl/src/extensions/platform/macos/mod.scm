(library
  (scaffold extensions platform macos)
  (export
    macos/command-tool
    macos/xcode-command-line-tools-platform
    macos/zip-app-bin-platform
    xcode-command-line-tools-platform
    zip-app-bin-platform)
  (import (rnrs) (scaffold catalog base) (scaffold extensions platform macos base))

  (moduledoc
    (summary "macOS platform helpers for declaring required commands and installer platforms.")
    (group "macOS tools")))
