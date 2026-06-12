(import (rnrs) (scaffold config) (scaffold catalog) (scaffold host))

(object
  (field 'os host/os)
  (field 'arch host/arch)
  (field 'platform host/platform)
  (field 'commands command/available)
  (field 'has-shell (command/available? "sh"))
  (field 'shell-path (command/path "sh"))
  (field 'has-shell-path (command/path? "sh"))
  (field
    'missing-command-path
    (command/path "DEFINITELY_NOT_A_REAL_SCAFFOLD_TEST_COMMAND"))
  (field
    'missing-command-path?
    (command/path? "DEFINITELY_NOT_A_REAL_SCAFFOLD_TEST_COMMAND"))
  (field 'matches-os (host/matches? host/os))
  (field 'matches-platform (host/matches? host/platform))
  (field 'matches-predicate (host/matches? (predicate host/os host/arch)))
  (field 'matches-wrong-os (host/matches? (if (eq? host/os 'linux) 'windows 'linux)))
  (field 'matches-invalid (host/matches? 42))
  (field 'has-path-env (env/var? "PATH"))
  (field 'missing-env (env/var "DEFINITELY_NOT_A_REAL_SCAFFOLD_TEST_ENV")))

(moduledoc (summary "Fixture for the generated host library values injected by Rust."))
