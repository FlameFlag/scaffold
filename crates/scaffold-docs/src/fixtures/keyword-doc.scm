(tool #:name "demo")
(doc 'local-helper
  (signature "(local-helper value)")
  (summary "Project-local docs.")
  (param 'value "Input value."))
(define (local-helper value) value)
