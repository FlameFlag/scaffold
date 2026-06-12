(import (acme tools))

(acme-tool "demo")

(acme-helper "demo")

(define (local-helper value) value)

(local-helper "demo")
