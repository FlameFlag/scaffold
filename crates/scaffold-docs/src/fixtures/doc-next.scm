(moduledoc (summary "Module docs.") (group "Fixtures") (effect 'pure))
(doc-next
  (summary "Create a demo.")
  (param 'name "Name for the demo."))
(define (demo name) name)
