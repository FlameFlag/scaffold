(library
  (scaffold test)
  (export assert/true assert/equal)
  (import (rnrs) (scaffold config))

  (doc-next
    (summary "Assert that a value is truthy.")
    (param 'value "Expression result that must not be `#f`.")
    (returns "`#t` when the assertion passes.")
    (markdown
      "Raises an assertion violation named `assert/true` when the value is false."))

  (define (assert/true value)
    (if value #t (assertion-violation 'assert/true "expected true")))

  (doc-next
    (summary "Assert that two Scheme values are equal.")
    (param 'expected "Expected value.")
    (param 'actual "Actual value produced by the code under test.")
    (returns "`#t` when the values compare with `equal?`.")
    (markdown "Raises an assertion violation containing both values when they differ."))

  (define (assert/equal expected actual)
    (if (equal? expected actual)
      #t
      (assertion-violation 'assert/equal "expected equal values" expected actual)))

  (moduledoc
    (summary "Tiny assertion helpers for DSL fixture and extension tests.")
    (group "Testing")))
