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
    (merged
      (object/merge
        base
        (object (field 'phase 'overridden))
        (object (field 'extra "kept"))))
    (updated
      (object/update-field
        base
        'name
        (lambda (name) (string-append name "-updated"))))
    (missing-update
      (object/update-field
        base
        'missing
        (lambda (value) "unexpected")))
    (mapped
      (object/map-vector-field
        base
        'bins
        (lambda (bin-name) (string-append bin-name "-mapped"))))
    (trimmed (object/remove-fields overridden 'phase))
    (split (split-fields (list "--locked" (field 'bin "demo") "--force")))
    (callback-split
      (call-with-split-fields
        (list "--features" (field 'mode "callback") "--locked")
        (lambda (flags fields)
          (object
            (field 'flags (apply arr flags))
            (field 'field-name (field/name (car fields)))
            (field 'field-value (field/value (car fields))))))))
  (object
    (field 'name (object/ref trimmed 'name))
    (field 'phase (object/ref trimmed 'phase 'missing))
    (field 'bins (object/ref trimmed 'bins))
    (field 'platforms (object/ref trimmed 'platforms))
    (field 'merged-phase (object/ref merged 'phase))
    (field 'merged-extra (object/ref merged 'extra))
    (field 'updated-name (object/ref updated 'name))
    (field 'missing-update (object/ref missing-update 'missing 'absent))
    (field 'mapped-bins (object/ref mapped 'bins))
    (field 'has-name (object/has-field? trimmed 'name))
    (field 'has-phase (object/has-field? trimmed 'phase))
    (field 'field-predicate (field? (field 'bin "demo")))
    (field 'split-flags (list->vector (car split)))
    (field 'split-field-name (field/name (car (cdr split))))
    (field 'split-field-value (field/value (car (cdr split))))
    (field 'callback-flags (object/ref callback-split 'flags))
    (field 'callback-field-name (object/ref callback-split 'field-name))
    (field 'callback-field-value (object/ref callback-split 'field-value))
    (field 'appended-vector (vector/append (arr "a") (arr "b" "c")))
    (field 'empty-vector (vector/append))))

(moduledoc
  (summary "Fixture for object merge, replacement, and vector append helpers."))
