(library
  (scaffold fs)
  (export path/exists? file/exists? directory/exists?)
  (import (rnrs) (scaffold config) (scaffold fs builtins))

  (extern-doc path/exists?
    (signature "(path/exists? path)")
    (summary "Return whether an absolute path exists on the host filesystem.")
    (param 'path "Absolute path string to inspect.")
    (returns "`#t` when the path exists, otherwise `#f`."))

  (define path/exists? %path/exists?)

  (extern-doc file/exists?
    (signature "(file/exists? path)")
    (summary "Return whether an absolute path exists and is a regular file.")
    (param 'path "Absolute path string to inspect.")
    (returns "`#t` when the path exists as a file, otherwise `#f`."))

  (define file/exists? %file/exists?)

  (extern-doc directory/exists?
    (signature "(directory/exists? path)")
    (summary "Return whether an absolute path exists and is a directory.")
    (param 'path "Absolute path string to inspect.")
    (returns "`#t` when the path exists as a directory, otherwise `#f`."))

  (define directory/exists? %directory/exists?)

  (moduledoc
    (summary "Read-only filesystem predicates backed by the Rust host runtime.")
    (group "Filesystem")
    (effect 'host-read-only)
    (requires-capability 'scaffold.fs)))
