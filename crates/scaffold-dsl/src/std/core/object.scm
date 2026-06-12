(library
  (scaffold core object)
  (export
    field
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
    object/append-field-vector
    object/append-vector)
  (import (rnrs) (scaffold core vector))

  (define field cons)

  (define field/name car)

  (define field/value cdr)

  (define (object . fields) fields)

  (define (field-key=? left right) (equal? left right))

  (define (field-name-member? name names)
    (cond
      ((null? names) #f)
      ((field-key=? name (car names)) #t)
      (else (field-name-member? name (cdr names)))))

  (define (object/has-field? obj name) (if (assoc name obj) #t #f))

  (define (remove-field-names fields names)
    (cond
      ((null? fields) '())
      ((field-name-member? (field/name (car fields)) names)
        (remove-field-names (cdr fields) names))
      (else (cons (car fields) (remove-field-names (cdr fields) names)))))

  (define (object/ref obj name . default)
    (let ((entry (assoc name obj)))
      (cond (entry (field/value entry)) ((null? default) #f) (else (car default)))))

  (define (object/remove-fields obj . names) (remove-field-names obj names))

  (define (object/replace-fields obj . fields)
    (append (remove-field-names obj (map field/name fields)) fields))

  (define (merge-one base override) (apply object/replace-fields base override))

  (define (object/merge base . overrides)
    (let loop
      ((result base) (rest overrides))
      (if (null? rest) result (loop (merge-one result (car rest)) (cdr rest)))))

  (define (object/inherit base . fields) (apply object/replace-fields base fields))

  (define (object/override base proc) (object/merge base (proc base)))

  (define (object/replace-field obj name value)
    (object/replace-fields obj (field name value)))

  (define (object/append-field-vector obj name values)
    (object/replace-field obj name (vector/append (object/ref obj name (arr)) values)))

  (define (object/append-vector obj name . values)
    (object/append-field-vector obj name (list->vector values))))
