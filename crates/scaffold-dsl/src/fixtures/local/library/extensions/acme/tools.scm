(library
  (acme tools)
  (export acme-tool)
  (import (rnrs) (scaffold catalog base) (scaffold host))

  (doc-next
    (signature "(acme-tool ...)")
    (summary "Create a package-backed Acme tool fixture."))

  (define (acme-tool name package-name)
    (tool
      name
      (package
        (field 'name package-name)
        (field 'install-argv (arr "custom-install" "{{ package }}")))
      (field 'has-cargo (command/available? "cargo"))))

  (moduledoc
    (summary "Fixture local extension library that exports an Acme tool constructor.")))
