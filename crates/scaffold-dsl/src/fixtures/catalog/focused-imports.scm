(import
  (rnrs)
  (scaffold catalog base)
  (scaffold test)
  (scaffold extensions ecosystem uv))

(define package-action
  (package (field 'name "demo") (field 'install-argv (arr "install" "{{ package }}"))))

(define package-demo (tool "package-demo" (install-argv/prepend package-action "sudo")))

(define ruff (uv/tool "ruff"))

(assert/equal "package-demo" (object/ref package-demo 'name))

(assert/equal
  (arr "sudo" "install" "{{ package }}")
  (object/ref (object/ref package-demo 'action) 'install-argv))

(assert/equal "ruff" (object/ref ruff 'name))

(moduledoc (summary "Assertions for focused catalog module imports."))

(extern-doc package-action
  (signature "(package-action ...)")
  (summary "Package action fixture used to verify focused action imports."))

(extern-doc package-demo
  (signature "(package-demo ...)")
  (summary "Tool fixture backed by a package action."))

(extern-doc ruff
  (signature "(ruff ...)")
  (summary "Required-tool fixture for focused catalog module assertions."))
