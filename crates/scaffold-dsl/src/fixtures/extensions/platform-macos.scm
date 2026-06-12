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

(moduledoc (summary "Fixture for macOS platform helpers."))
