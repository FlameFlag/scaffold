(library
  (examples tools builds)
  (export hello-example build/tools)
  (import (rnrs) (scaffold catalog))

  (moduledoc (summary "Workspace source build examples.") (group "Examples"))

  (doc-next (summary "Tiny C program built from a workspace-relative source path."))

  (define hello-example
    (tool
      "hello-example"
      (workspace/build
        "sources/hello"
        (field
          'argvs
          (arr (arr "sh" "-lc" "cc hello.c -o {{ bin_dir }}/hello-example"))))
      (depends "git")
      (field 'bins (arr (bin "hello-example")))
      (field 'checks (arr (check (arr "hello-example"))))
      (uninstall/paths "{{ bin_dir }}/hello-example")
      (meta
        (description "Minimal workspace build example.")
        (license "MIT")
        (tags "build" "workspace")
        (main-program "hello-example"))))

  (doc-next (summary "Return workspace build examples."))

  (define (build/tools) (list hello-example)))
