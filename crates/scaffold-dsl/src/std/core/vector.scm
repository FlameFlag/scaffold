(library
  (scaffold core vector)
  (export arr list->arr vector/map vector/append arr/append-list arr/prepend-list)
  (import (rnrs))

  (define arr vector)

  (define list->arr list->vector)

  (define vector/map vector-map)

  (define (append-vector-lists vectors)
    (fold-right append '() (map vector->list vectors)))

  (define (vector/append . vectors) (list->arr (append-vector-lists vectors)))

  (define (arr/append-list values suffix) (vector/append values (list->arr suffix)))

  (define (arr/prepend-list prefix values) (vector/append (list->arr prefix) values)))
