(library
  (scaffold extensions ecosystem go)
  (export go/install-argv go/install-platform go/tool)
  (import (rnrs) (scaffold catalog base))

  (doc-next
    (signature "(go/install-argv package ...)")
    (summary "Build argv for `go install` into the Scaffold bin directory.")
    (param 'package "Go package specs or local command paths passed to `go install`.")
    (returns "Vector argv using `CGO_ENABLED=0` and `GOBIN={{ bin_dir }}`."))

  (doc-next
    (hidden)
    (summary "Build Go install argv from an existing package list."))

  (define (go/install-argv-list packages)
    (arr/append-list
      (arr "env" "CGO_ENABLED=0" "GOBIN={{ bin_dir }}" "go" "install")
      packages))

  (define (go/install-argv . packages)
    (go/install-argv-list packages))

  (doc-next
    (signature "(go/install-platform predicate name package-or-field ...)")
    (summary "Create a package/platform override that installs one or more Go packages.")
    (param 'predicate "Host predicate for this package rule.")
    (param 'name "Platform rule name.")
    (param
      'package-or-field
      "Go package specs or local command paths followed by optional platform fields."))

  (define (go/install-platform predicate-value name . options)
    (call-with-split-fields
      options
      (lambda (packages fields)
        (object/merge
          (package/platform
            predicate-value
            (arr "env" "go")
            (go/install-argv-list packages)
            (field 'name name))
          fields))))

  (doc-next
    (signature "(go/tool name package bin-name field ...)")
    (summary "Create a catalog tool installed by `go install`.")
    (param 'name "Catalog tool name.")
    (param 'package "Go package spec passed to `go install`.")
    (param 'bin-name "Executable produced by the package.")
    (param 'field "Additional tool fields that override defaults."))

  (define (go/tool name package-name bin-name . fields)
    (object/merge
      (tool
        name
        (package
          (field 'name package-name)
          (field 'install-argv (go/install-argv "{{ package }}")))
        (field 'bins (arr (bin bin-name)))
        (uninstall/paths (string-append "{{ bin_dir }}/" bin-name)))
      fields))

  (moduledoc
    (summary "Go helpers for tools installed from modules or local command paths.")
    (group "Go")))
