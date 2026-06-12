(import (rnrs) (scaffold config))

(moduledoc (summary "Fixture for a locally scoped syntax-rules macro."))

(let-syntax
  ((named-command
     (syntax-rules
       ()
       ((_ name program arg ...)
         (list (cons "name" name) (cons "argv" (list->vector (list program arg ...))))))))
  (named-command "demo" "cargo" "test" "--quiet"))
