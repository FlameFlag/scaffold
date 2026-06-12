(import
  (rnrs)
  (scaffold catalog)
  (scaffold test)
  (scaffold extensions app winget)
  (scaffold extensions build cmake)
  (scaffold extensions build gcc)
  (scaffold extensions build make))

(doc-next
  (signature "(winget-demo ...)")
  (summary "WinGet demo tool used by extension helper assertions."))

(define winget-demo (winget/package "ripgrep" "BurntSushi.ripgrep.MSVC" "rg"))

(doc-next (summary "Fixture WinGet platform override."))

(define winget-platform
  (winget/package-platform "BurntSushi.ripgrep.MSVC" "--scope" "machine"))

(doc-next (summary "Fixture WinGet platform override with extra fields."))

(define winget-field-platform
  (winget/package-platform
    "Microsoft.VisualStudioCode"
    (field
      'install-argvs
      (arr (arr "winget" "source" "update") (winget/install-argv "{{ package }}")))))

(assert/equal
  (arr
    "winget"
    "install"
    "--id"
    "{{ package }}"
    "--exact"
    "--silent"
    "--accept-package-agreements"
    "--accept-source-agreements")
  (object/ref (object/ref winget-demo 'action) 'install-argv))

(assert/equal
  (arr "winget" "list" "--id" "{{ package }}" "--exact")
  (object/ref (vector-ref (object/ref winget-demo 'checks) 0) 'argv))

(assert/equal
  (arr "winget" "uninstall" "--id" "{{ package }}" "--exact" "--silent")
  (object/ref
    (vector-ref (object/ref (object/ref winget-demo 'uninstall) 'commands) 0)
    'argv))

(assert/equal "BurntSushi.ripgrep.MSVC" (object/ref winget-platform 'name))

(assert/equal
  (arr
    "winget"
    "install"
    "--id"
    "{{ package }}"
    "--exact"
    "--silent"
    "--accept-package-agreements"
    "--accept-source-agreements"
    "--scope"
    "machine")
  (object/ref winget-platform 'install-argv))

(assert/equal
  (arr "winget" "source" "update")
  (vector-ref (object/ref winget-field-platform 'install-argvs) 0))

(assert/equal
  (arr "gcc" "-O2" "hello.c" "-o" "hello")
  (gcc/compile-argv "hello" (arr "hello.c") (arr "-O2")))

(assert/equal
  (arr "cmake" "-S" "." "-B" "build" "-G" "Ninja")
  (cmake/configure-argv "." "build" "-G" "Ninja"))

(assert/equal (arr "make" "-j" "8" "install") (make/jobs-argv "8" "install"))

(doc-next
  (signature "(cmake-demo ...)")
  (summary "CMake demo tool used by extension helper assertions."))

(define cmake-demo (cmake/project-tool "hello" "vendor/hello"))

(assert/equal 3 (vector-length (object/ref (object/ref cmake-demo 'action) 'argvs)))

(assert/equal "hello" (object/ref (vector-ref (object/ref cmake-demo 'bins) 0) 'name))

(doc-next
  (signature "(make-demo ...)")
  (summary "Make demo tool used by extension helper assertions."))

(define make-demo (make/project-tool "hello-make" "vendor/hello-make"))

(assert/equal
  (arr "make" "PREFIX={{ prefix }}" "install")
  (vector-ref (object/ref (object/ref make-demo 'action) 'argvs) 1))

(assert/equal
  "hello-make"
  (object/ref (vector-ref (object/ref make-demo 'bins) 0) 'name))

(moduledoc (summary "Fixture for WinGet, GCC, CMake, and Make extension helpers."))
