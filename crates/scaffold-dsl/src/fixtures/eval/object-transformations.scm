(import (rnrs) (scaffold config))

(let*
  ((base
     (object (field 'name "demo") (field 'phase 'packages) (field 'bins (arr "demo"))))
    (with-bin (object/append-vector base 'bins "democtl"))
    (variant
      (object/replace-fields
        with-bin
        (field 'phase 'builds)
        (field 'platforms (arr 'linux))))
    (overridden
      (object/override
        variant
        (lambda (old)
          (object (field 'name (string-append (object/ref old 'name) "-override"))))))
    (trimmed (object/remove-fields overridden 'phase)))
  (object
    (field 'name (object/ref trimmed 'name))
    (field 'phase (object/ref trimmed 'phase 'missing))
    (field 'bins (object/ref trimmed 'bins))
    (field 'platforms (object/ref trimmed 'platforms))))

(moduledoc
  (summary "Fixture for object merge, replacement, and vector append helpers."))
