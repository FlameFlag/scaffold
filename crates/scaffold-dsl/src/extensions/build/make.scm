(library
  (scaffold extensions build make)
  (export make/argv make/jobs-argv make/prefix-install-argv make/tool make/project-tool)
  (import (rnrs) (scaffold catalog base))

  (doc-next
    (signature "(make/argv argv ...)")
    (summary "Build a raw Make argv vector."))

  (define (make/argv . argv) (arr/append-list (arr "make") argv))

  (doc-next
    (signature "(make/jobs-argv jobs argv ...)")
    (summary "Build argv for `make -j <jobs>`."))

  (define (make/jobs-argv jobs . argv) (arr/append-list (arr "make" "-j" jobs) argv))

  (doc-next
    (signature "(make/prefix-install-argv prefix target ...)")
    (summary "Build argv for `make PREFIX=<prefix> install` or custom targets."))

  (define make/prefix-install-argv
    (case-lambda
      ((prefix) (arr "make" (string-append "PREFIX=" prefix) "install"))
      ((prefix . targets)
        (arr/append-list (arr "make" (string-append "PREFIX=" prefix)) targets))))

  (doc-next
    (signature "(make/tool field ...)")
    (summary "Create a required host Make tool descriptor."))

  (define (make/tool . fields)
    (object/merge
      (tool "make" (required) (field 'bins (arr (bin/version "make"))))
      fields))

  (doc-next
    (signature "(make/project-tool name path field ...)")
    (summary
      "Create a build action that runs `make` and `make PREFIX={{ prefix }} install`.")
    (param 'name "Catalog tool name and default installed binary name.")
    (param 'path "Source path stored on the build action.")
    (param 'field "Additional tool fields that override defaults."))

  (define (make/project-tool name path . fields)
    (object/merge
      (tool
        name
        (build
          (field 'path path)
          (field 'argvs (arr (make/argv) (make/prefix-install-argv "{{ prefix }}"))))
        (field 'bins (arr (bin name))))
      fields))

  (moduledoc
    (summary "GNU Make command builders and simple source build helpers.")
    (group "Build tools")))
