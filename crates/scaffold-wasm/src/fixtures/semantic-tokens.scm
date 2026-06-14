(define (local-helper value) value)
(if #t (tool #:name "demo") (local-helper "demo"))
(doc 'local-helper (summary "Docs"))
