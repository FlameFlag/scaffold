(library
  (scaffold extensions build cmake)
  (export
    cmake/argv
    cmake/configure-argv
    cmake/build-argv
    cmake/install-argv
    cmake/tool
    cmake/project-tool)
  (import (rnrs) (scaffold catalog base))

  (doc-next
    (signature "(cmake/argv argv ...)")
    (summary "Build a raw CMake argv vector."))

  (define (cmake/argv . argv) (arr/append-list (arr "cmake") argv))

  (doc-next
    (signature "(cmake/configure-argv source/dir build-dir flag ...)")
    (summary "Build argv for `cmake -S <source> -B <build>`."))

  (define (cmake/configure-argv source/dir build-dir . flags)
    (vector/append (arr "cmake" "-S" source/dir "-B" build-dir) (list->vector flags)))

  (doc-next
    (signature "(cmake/build-argv build-dir flag ...)")
    (summary "Build argv for `cmake --build <build-dir>`."))

  (define (cmake/build-argv build-dir . flags)
    (vector/append (arr "cmake" "--build" build-dir) (list->vector flags)))

  (doc-next
    (signature "(cmake/install-argv build-dir prefix flag ...)")
    (summary "Build argv for `cmake --install <build-dir> --prefix <prefix>`."))

  (define (cmake/install-argv build-dir prefix . flags)
    (vector/append
      (arr "cmake" "--install" build-dir "--prefix" prefix)
      (list->vector flags)))

  (doc-next
    (signature "(cmake/tool field ...)")
    (summary "Create a required host CMake tool descriptor."))

  (define (cmake/tool . fields)
    (apply tool "cmake" (required) (field 'bins (arr (bin/version "cmake"))) fields))

  (doc-next
    (signature "(cmake/project-tool name path field ...)")
    (summary
      "Create a build action for a CMake project using configure/build/install commands.")
    (param 'name "Catalog tool name and default installed binary name.")
    (param 'path "Source path stored on the build action.")
    (param 'field "Additional tool fields that override defaults."))

  (define (cmake/project-tool name path . fields)
    (let ((build-dir (string-append "{{ source_dir }}/build-scaffold")))
      (apply
        tool
        name
        (build
          (field 'path path)
          (field
            'argvs
            (arr
              (cmake/configure-argv
                "{{ source_dir }}"
                build-dir
                "-DCMAKE_BUILD_TYPE=Release"
                "-DCMAKE_INSTALL_PREFIX={{ prefix }}")
              (cmake/build-argv build-dir "--config" "Release")
              (cmake/install-argv build-dir "{{ prefix }}" "--config" "Release"))))
        (field 'bins (arr (bin name)))
        fields)))

  (moduledoc
    (summary "CMake configure, build, and install helpers.")
    (group "Build tools")))
