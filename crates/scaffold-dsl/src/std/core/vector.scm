(library
  (scaffold core vector)
  (export arr vector/map vector/append arr/append-list arr/prepend-list)
  (import (rnrs))

  (define (arr . values) (list->vector values))

  (define (vector/map proc values) (list->vector (map proc (vector->list values))))

  (define (append-vector-lists vectors)
    (if (null? vectors)
      '()
      (append (vector->list (car vectors)) (append-vector-lists (cdr vectors)))))

  (define (vector/append . vectors) (list->vector (append-vector-lists vectors)))

  (define (arr/append-list values suffix) (vector/append values (list->vector suffix)))

  (define (arr/prepend-list prefix values) (vector/append (list->vector prefix) values)))
