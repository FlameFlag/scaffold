(moduledoc (summary "Path docs.") (group "Paths") (requires-capability 'scaffold.path))
(extern-doc path/join
  (signature (path/join first part ...))
  (summary "Join path components."))
(define path/join %path/join)
(extern-doc path/separator
  (signature value path/separator)
  (summary "Host path separator."))
(define path/separator %path/separator)
