(import (rnrs) (scaffold config) (scaffold workspace) (scaffold catalog))

(object
  (field 'workspace-root workspace/root)
  (field 'source-path source/path)
  (field 'source-dir source/dir)
  (field 'workspace-file (workspace/path "nested" "file.txt")))

(tool
  "helper-demo"
  (workspace/build "vendor/demo" (field 'argv (arr "make" "install")))
  (tool/platforms 'linux 'macos)
  (uninstall/paths "{{ home }}/.helper-demo"))

(moduledoc (summary "Fixture for evaluator context workspace values."))
