(library
  (scaffold core object)
  (export
    field
    field?
    field/name
    field/value
    object
    object/has-field?
    object/ref
    object/remove-fields
    object/replace-fields
    object/merge
    object/inherit
    object/override
    object/replace-field
    object/update-field
    object/map-vector-field
    object/append-field-vector
    object/append-vector
    split-fields
    call-with-split-fields)
  (import (rnrs) (scaffold core vector))

  (define field cons)

  (define field? pair?)

  (define field/name car)

  (define field/value cdr)

  (define object list)

  (define (field-name-member? name names)
    (and (member name names) #t))

  (define (object/has-field? obj name) (and (assoc name obj) #t))

  (define (remove-field-names fields names)
    (remp
      (lambda (field-value) (field-name-member? (field/name field-value) names))
      fields))

  (define (object/ref-default obj name default)
    (let ((entry (assoc name obj)))
      (if entry (field/value entry) default)))

  (define object/ref
    (case-lambda
      ((obj name) (object/ref-default obj name #f))
      ((obj name default) (object/ref-default obj name default))
      ((obj name default . _) (object/ref-default obj name default))))

  (define (object/remove-fields obj . names) (remove-field-names obj names))

  (define (object/replace-field-list obj fields)
    (append (remove-field-names obj (map field/name fields)) fields))

  (define (object/replace-fields obj . fields)
    (object/replace-field-list obj fields))

  (define (merge-one base override) (object/replace-field-list base override))

  (define (object/merge base . overrides) (fold-left merge-one base overrides))

  (define object/inherit object/replace-fields)

  (define (object/override base proc) (object/merge base (proc base)))

  (define (object/replace-field obj name value)
    (object/replace-fields obj (field name value)))

  (define (object/update-field obj name proc)
    (if (object/has-field? obj name)
      (object/replace-field obj name (proc (object/ref obj name)))
      obj))

  (define (object/map-vector-field obj name proc)
    (object/update-field obj name (lambda (values) (vector/map proc values))))

  (define (object/append-field-vector obj name values)
    (object/replace-field obj name (vector/append (object/ref obj name (arr)) values)))

  (define (object/append-vector obj name . values)
    (object/append-field-vector obj name (list->arr values)))

  (define (split-fields values)
    (call-with-values
      (lambda () (partition field? values))
      (lambda (fields others) (cons others fields))))

  (define (call-with-split-fields values proc)
    (call-with-values
      (lambda () (partition field? values))
      (lambda (fields others) (proc others fields)))))
