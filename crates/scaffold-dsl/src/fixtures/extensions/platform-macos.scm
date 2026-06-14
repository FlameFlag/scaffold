(import (rnrs) (scaffold catalog) (scaffold test) (scaffold extensions platform macos))

(doc-next
  (signature "(macos-required ...)")
  (summary "Required macOS command descriptor used by macOS extension assertions."))

(define macos-required (macos/command-tool "xcrun"))

(assert/equal "xcrun" (object/ref macos-required 'name))

(assert/equal 'required (object/ref (object/ref macos-required 'action) 'type))

(assert/equal 'macos (vector-ref (object/ref macos-required 'platforms) 0))

(assert/equal
  "xcrun"
  (object/ref (vector-ref (object/ref macos-required 'bins) 0) 'name))

(define xcode-platform (macos/xcode-command-line-tools-platform))

(assert/equal 'macos (object/ref xcode-platform 'when))

(assert/equal
  (arr "sh" "-c" "xcode-select -p >/dev/null 2>&1 || xcode-select --install")
  (object/ref xcode-platform 'install-argv))

(define app-platform
  (macos/zip-app-bin-platform
    'macos
    "demo"
    "https://example.test/demo.zip"
    "demo.zip"
    "Demo.app"
    "Contents/Resources/bin/demo"
    "demo"))

(assert/equal
  (arr "curl" "ditto" "ln" "mkdir" "rm")
  (object/ref app-platform 'requires-commands))

(assert/equal
  (arr "mkdir" "-p" "{{ state_dir }}/tools/demo/latest" "{{ bin_dir }}")
  (vector-ref (object/ref app-platform 'install-argvs) 0))

(assert/equal 9 (vector-length (object/ref app-platform 'install-argvs)))

(assert/equal
  (arr "rm" "-rf" "{{ state_dir }}/tools/demo/latest/extract")
  (vector-ref (object/ref app-platform 'install-argvs) 6))

(assert/equal
  (arr "rm" "-f" "{{ state_dir }}/tools/demo/latest/demo.zip")
  (vector-ref (object/ref app-platform 'install-argvs) 7))

(moduledoc (summary "Fixture for macOS platform helpers."))
