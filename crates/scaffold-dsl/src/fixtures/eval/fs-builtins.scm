(import (rnrs) (scaffold config) (scaffold fs) (scaffold workspace))

(object
  (field 'source-exists (path/exists? source/path))
  (field 'source-file (file/exists? source/path))
  (field 'source-dir (directory/exists? source/path))
  (field 'root-exists (path/exists? workspace/root))
  (field 'root-dir (directory/exists? workspace/root))
  (field 'missing-exists (path/exists? (workspace/path "missing-file.txt")))
  (field 'missing-file (file/exists? (workspace/path "missing-file.txt"))))

(moduledoc (summary "Fixture for read-only filesystem predicates."))
