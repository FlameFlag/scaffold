(library
  (scaffold path)
  (export
    path/join
    path/normalize
    path/parent
    path/file-name
    path/extension
    path/absolute?
    path/relative?
    path/separator)
  (import (rnrs) (scaffold config) (scaffold path builtins))

  (extern-doc path/join
    (signature "(path/join first part ...)")
    (summary "Join path components using the current host path rules.")
    (param 'first "First path component.")
    (param 'part "Additional path components.")
    (returns "A string path. Absolute later components may replace earlier components."))

  (define path/join %path/join)

  (extern-doc path/normalize
    (signature "(path/normalize path)")
    (summary "Lexically normalize a path without reading the filesystem.")
    (param 'path "Path string to normalize.")
    (returns "A normalized path string."))

  (define path/normalize %path/normalize)

  (extern-doc path/parent
    (signature "(path/parent path)")
    (summary "Return the parent directory of a path.")
    (param 'path "Path string to inspect.")
    (returns "The parent path as a string, or `#f` when no parent exists."))

  (define path/parent %path/parent)

  (extern-doc path/file-name
    (signature "(path/file-name path)")
    (summary "Return the final file name component of a path.")
    (param 'path "Path string to inspect.")
    (returns "The file name as a string, or `#f` when no file name exists."))

  (define path/file-name %path/file-name)

  (extern-doc path/extension
    (signature "(path/extension path)")
    (summary "Return the final extension component of a path.")
    (param 'path "Path string to inspect.")
    (returns "The extension as a string without a leading dot, or `#f`."))

  (define path/extension %path/extension)

  (extern-doc path/absolute?
    (signature "(path/absolute? path)")
    (summary "Return whether a path is absolute on the current host.")
    (param 'path "Path string to inspect."))

  (define path/absolute? %path/absolute?)

  (extern-doc path/relative?
    (signature "(path/relative? path)")
    (summary "Return whether a path is relative on the current host.")
    (param 'path "Path string to inspect."))

  (define path/relative? %path/relative?)

  (extern-doc path/separator
    (signature "path/separator")
    (summary "Directory separator string for paths on the current host."))

  (define path/separator (%path/separator))

  (moduledoc
    (summary "Lexical path helpers backed by the Rust host runtime.")
    (group "Paths")
    (effect 'pure)
    (requires-capability 'scaffold.path)))
