(import (rnrs) (scaffold catalog))

(define-syntax local-required-tool
  (syntax-rules
    ()
    ((_ name)
      (list
        (cons "name" name)
        (cons
          "bins"
          (vector
            (list (cons "name" name) (cons "version_argv" (vector name "--version")))))
        (cons "action" (list (cons "type" "required")))))))

(catalog
  (required/tool
    "library-macro"
    (field 'paths (arr (tool/path 'macos "/tmp/scaffold"))))
  (local-required-tool "local-macro"))

(moduledoc
  (summary "Fixture for catalog macro expansion with locally defined required tools."))

(extern-doc local-required-tool
  (signature "(local-required-tool ...)")
  (summary "Local macro wrapper that emits a required/tool catalog entry."))
