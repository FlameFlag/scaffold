(library
  (scaffold workspace)
  (export workspace/root source/path source/dir workspace/path)
  (import (rnrs) (scaffold config) (scaffold path))

  (extern-doc workspace/root
    (signature "workspace/root")
    (summary "Root directory used for the current Scaffold evaluation.")
    (returns "A path string, or `#f` when no workspace root is available."))

  (define (injected-path present value)
    (and (string=? present "#t") value))

  (define workspace/root
    (injected-path "{{ workspace_root_present }}" "{{ workspace_root }}"))

  (extern-doc source/path
    (signature "source/path")
    (summary "Path of the Scheme source currently being evaluated.")
    (returns "A path string, or `#f` when no source path is available."))

  (define source/path
    (injected-path "{{ source_path_present }}" "{{ source_path }}"))

  (extern-doc source/dir
    (signature "source/dir")
    (summary "Directory of the Scheme source currently being evaluated.")
    (returns "A path string, or `#f` when no source path is available."))

  (define source/dir (and source/path (path/parent source/path)))

  (doc-next
    (hidden)
    (summary "Join a root path with an existing list of path components."))

  (define (workspace/path-list root parts) (fold-left path/join root parts))

  (doc-next
    (summary "Join path components under `workspace/root`.")
    (param 'part "Path component appended under the workspace root.")
    (returns "A path string under `workspace/root`."))

  (define (workspace/path . parts)
    (if workspace/root
      (workspace/path-list workspace/root parts)
      (assertion-violation 'workspace/path "workspace/root is unavailable")))

  (moduledoc
    (summary "Read-only workspace facts injected by the Rust evaluator context.")
    (group "Workspace")
    (effect 'context-read-only)
    (requires-capability 'scaffold.workspace)))
