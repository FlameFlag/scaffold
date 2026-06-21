(library
  (examples tools ecosystem)
  (export prettier ruff ecosystem/tools)
  (import
    (rnrs)
    (scaffold catalog)
    (scaffold extensions ecosystem bun)
    (scaffold extensions ecosystem uv))

  (moduledoc
    (summary "Language ecosystem package manager examples.")
    (group "Examples"))

  (doc-next (summary "Prettier installed globally with Bun."))

  (define prettier
    (bun/global-tool
      "prettier"
      "prettier"
      "prettier"
      (depends "bun")
      (meta
        (description "Code formatter installed with Bun.")
        (home-page "https://prettier.io/")
        (license "MIT")
        (tags "javascript" "formatter")
        (main-program "prettier"))))

  (doc-next (summary "Ruff installed as an isolated Python tool with uv."))

  (define ruff
    (uv/tool
      "ruff"
      (depends "uv")
      (meta
        (description "Python linter and formatter installed with uv.")
        (home-page "https://docs.astral.sh/ruff/")
        (license "MIT")
        (tags "python" "linter" "formatter")
        (main-program "ruff"))))

  (doc-next (summary "Return ecosystem-managed example tools."))

  (define (ecosystem/tools) (list prettier ruff)))
