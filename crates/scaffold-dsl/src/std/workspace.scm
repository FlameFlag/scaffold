(library
  (scaffold workspace)
  (export workspace/root source/path source/dir workspace/path)
  (import (rnrs) (scaffold config) (scaffold path))

  (extern-doc workspace/root
    (signature "workspace/root")
    (summary "Root directory used for the current Scaffold evaluation.")
    (returns "A path string, or `#f` when no workspace root is available."))

  (define workspace/root @WORKSPACE_ROOT@)

  (extern-doc source/path
    (signature "source/path")
    (summary "Path of the Scheme source currently being evaluated.")
    (returns "A path string, or `#f` when no source path is available."))

  (define source/path @SOURCE_PATH@)

  (extern-doc source/dir
    (signature "source/dir")
    (summary "Directory of the Scheme source currently being evaluated.")
    (returns "A path string, or `#f` when no source path is available."))

  (define source/dir @SOURCE_DIR@)

  (doc-next
    (summary "Join path components under `workspace/root`.")
    (param 'part "Path component appended under the workspace root.")
    (returns "A path string under `workspace/root`."))

  (define (workspace/path . parts)
    (if workspace/root
      (apply path/join workspace/root parts)
      (assertion-violation 'workspace/path "workspace/root is unavailable")))

  (moduledoc
    (summary "Read-only workspace facts injected by the Rust evaluator context.")
    (group "Workspace")
    (effect 'context-read-only)
    (requires-capability 'scaffold.workspace)))
