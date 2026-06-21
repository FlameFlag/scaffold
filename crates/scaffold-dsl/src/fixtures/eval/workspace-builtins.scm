(import (rnrs) (scaffold config) (scaffold workspace) (scaffold catalog))

(object
  (field 'workspace-root workspace/root)
  (field 'source-path source/path)
  (field 'source-dir source/dir)
  (field 'workspace-file (workspace/path "nested" "file.txt")))

(tool
  "helper-demo"
  (workspace/build "vendor/demo" (field 'argv (arr "make" "install")))
  (tool/bins (bin/version "helper-demo" "--version"))
  (tool/checks (host/check 'linux (arr "test" "-x" "{{ bin_dir }}/helper-demo")))
  (tool/paths (tool/path 'macos "/Applications/Helper Demo.app"))
  (tool/platforms 'linux 'macos)
  (tool/skip-verify-after-install)
  (uninstall/paths "{{ home }}/.helper-demo"))

(moduledoc (summary "Fixture for evaluator context workspace values."))
