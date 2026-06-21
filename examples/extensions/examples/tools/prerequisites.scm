(library
  (examples tools prerequisites)
  (export git bun uv prerequisite/tools)
  (import (rnrs) (scaffold catalog))

  (moduledoc
    (summary "Required host commands used by other example tools.")
    (group "Examples"))

  (doc-next (summary "Required Git command."))

  (define git
    (tool
      "git"
      (required)
      (field 'bins (arr (bin/version "git" "--version")))
      (field 'checks (arr (check (arr "git" "--version"))))
      (meta
        (description "Git must already be present before source-oriented tools run.")
        (home-page "https://git-scm.com/")
        (license "GPL-2.0-only")
        (tags "vcs" "required")
        (main-program "git"))))

  (doc-next (summary "Required Bun command for JavaScript ecosystem tools."))

  (define bun
    (tool
      "bun"
      (required)
      (field 'bins (arr (bin/version "bun" "--version")))
      (field 'checks (arr (check (arr "bun" "--version"))))
      (meta
        (description "Bun runtime used by the JavaScript ecosystem examples.")
        (home-page "https://bun.sh/")
        (license "MIT")
        (tags "javascript" "required")
        (main-program "bun"))))

  (doc-next (summary "Required uv command for Python ecosystem tools."))

  (define uv
    (tool
      "uv"
      (required)
      (field 'bins (arr (bin/version "uv" "--version") (bin "uvx")))
      (field 'checks (arr (check (arr "uv" "--version"))))
      (meta
        (description "uv package and tool manager used by the Python examples.")
        (home-page "https://docs.astral.sh/uv/")
        (license "MIT OR Apache-2.0")
        (tags "python" "required")
        (main-program "uv"))))

  (doc-next (summary "Return prerequisite tools for the examples catalog."))

  (define (prerequisite/tools) (list git bun uv)))
