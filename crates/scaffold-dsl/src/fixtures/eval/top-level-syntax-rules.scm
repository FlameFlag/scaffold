(import (rnrs) (scaffold config))

(moduledoc (summary "Fixture for a top-level syntax-rules macro."))

(define-syntax named-command
  (syntax-rules
    ()
    ((_ name program arg ...)
      (list (cons "name" name) (cons "argv" (list->vector (list program arg ...)))))))

(extern-doc named-command
  (signature "(named-command name program arg ...)")
  (summary "Build a command object with a name and argv vector."))

(named-command "demo" "cargo" "test" "--quiet")
