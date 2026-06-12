(library
  (software-packaging support shell)
  (export shell-command)
  (import (rnrs) (scaffold catalog))

  (doc-next (summary "Return an argv vector that runs a shell command."))

  (define (shell-command command) (arr "sh" "-lc" command)))
