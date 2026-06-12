(library
  (acme base)
  (export base-tool)
  (import (rnrs) (scaffold catalog base))

  (doc 'base-tool
    (signature "(base-tool ...)")
    (summary "Base Acme helper that creates a package-backed tool fixture."))

  (define (base-tool name package-name)
    (tool
      name
      (package
        (field 'name package-name)
        (field 'install-argv (arr "install-from-base" "{{ package }}")))))

  (moduledoc
    (summary "Fixture base library for local Acme extension dependency tests.")))
