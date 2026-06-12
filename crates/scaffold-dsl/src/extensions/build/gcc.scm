(library
  (scaffold extensions build gcc)
  (export gcc/argv gcc/compile-argv gcc/object-argv gcc/tool gcc/c-tool)
  (import (rnrs) (scaffold catalog base))

  (doc-next (signature "(gcc/argv argv ...)") (summary "Build a raw GCC argv vector."))

  (define (gcc/argv . argv) (arr/append-list (arr "gcc") argv))

  (doc-next
    (summary "Build argv for compiling and linking sources with GCC.")
    (param 'output "Output executable or object path.")
    (param 'sources "Vector of source files.")
    (param 'flags "Vector of GCC flags inserted before sources."))

  (define (gcc/compile-argv output sources flags)
    (vector/append (arr "gcc") flags sources (arr "-o" output)))

  (doc-next
    (signature "(gcc/object-argv source output flag ...)")
    (summary "Build argv for `gcc -c` object compilation."))

  (define (gcc/object-argv source output . flags)
    (vector/append (arr "gcc" "-c" source "-o" output) (list->vector flags)))

  (doc-next
    (signature "(gcc/tool field ...)")
    (summary "Create a required host GCC tool descriptor."))

  (define (gcc/tool . fields)
    (apply tool "gcc" (required) (field 'bins (arr (bin/version "gcc"))) fields))

  (doc-next
    (signature "(gcc/c-tool name path source field ...)")
    (summary
      "Create a build action that compiles one C source into the Scaffold prefix."))

  (define (gcc/c-tool name path source . fields)
    (apply
      tool
      name
      (build
        (field 'path path)
        (field
          'argv
          (gcc/compile-argv
            (string-append "{{ prefix }}/bin/" name)
            (arr (string-append "{{ source_dir }}/" source))
            (arr "-O2"))))
      (field 'bins (arr (bin name)))
      fields))

  (moduledoc
    (summary "GCC command builders and simple C source build helpers.")
    (group "Build tools")))
