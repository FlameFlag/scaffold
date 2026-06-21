(library
  (scaffold extensions ecosystem cargo)
  (export
    cargo/install-argv
    cargo/crate-install-argv
    cargo/uninstall-argv
    cargo/tool
    cargo/crate-tool
    cargo/crate-platform)
  (import (rnrs) (scaffold catalog base))

  (doc-next
    (summary "Build argv for `cargo install` into the Scaffold prefix.")
    (param 'source/path "Path passed to `cargo install --path`.")
    (param
      'extra-argv
      "Vector of extra cargo arguments appended after the default flags.")
    (returns "Vector argv using `--root {{ prefix }}`, `--force`, and `--locked`."))

  (define (cargo/install-argv source/path extra-argv)
    (vector/append
      (arr
        "cargo"
        "install"
        "--path"
        source/path
        "--root"
        "{{ prefix }}"
        "--force"
        "--locked")
      extra-argv))

  (doc-next
    (summary
      "Build argv for installing a published Cargo crate into the Scaffold prefix.")
    (param 'crate "Crate package spec passed to `cargo install`.")
    (param
      'extra-argv
      "Vector of extra cargo arguments appended after the default flags.")
    (returns "Vector argv using `--root {{ prefix }}`, `--force`, and `--locked`."))

  (define (cargo/crate-install-argv crate extra-argv)
    (vector/append
      (arr "cargo" "install" crate "--root" "{{ prefix }}" "--force" "--locked")
      extra-argv))

  (doc-next
    (summary
      "Build argv for uninstalling a Cargo-installed package from the Scaffold prefix.")
    (param 'package-name "Cargo package name passed to `cargo uninstall`.")
    (returns "Vector argv using `--root {{ prefix }}`."))

  (define (cargo/uninstall-argv package-name)
    (arr "cargo" "uninstall" "--root" "{{ prefix }}" package-name))

  (doc-next
    (signature "(cargo/tool name path field ...)")
    (summary "Create a catalog tool built with Cargo from a source path.")
    (param 'name "Catalog tool name.")
    (param 'path "Source path stored on the build action.")
    (param 'field "Additional tool fields that override defaults.")
    (returns "A tool with a build action that runs `cargo install`."))

  (define (cargo/tool name path . fields)
    (object/merge
      (tool
        name
        (build
          (field 'path path)
          (field 'argv (cargo/install-argv "{{ source_dir }}" (arr))))
        (field
          'uninstall
          (uninstall
            (field 'commands (arr (uninstall/command (cargo/uninstall-argv name)))))))
      fields))

  (doc-next
    (signature "(cargo/crate-tool name crate bin-name option ...)")
    (summary "Create a catalog tool installed from a published Cargo crate.")
    (param 'name "Catalog tool name.")
    (param 'crate "Crate package spec passed to `cargo install`.")
    (param 'bin-name "Executable exposed by the crate.")
    (param
      'option
      "Additional cargo install flags or tool fields. Field values are applied after defaults."))

  (define (cargo/crate-tool name crate bin-name . options)
    (call-with-split-fields
      options
      (lambda (flags fields)
        (object/merge
          (tool
            name
            (package
              (field 'name crate)
              (field
                'install-argv
                (cargo/crate-install-argv "{{ package }}" (arr/append-list (arr) flags))))
            (field 'bins (arr (bin bin-name)))
            (field
              'uninstall
              (uninstall
                (field
                  'commands
                  (arr (uninstall/command (cargo/uninstall-argv "{{ package }}")))))))
          fields))))

  (doc-next
    (signature "(cargo/crate-platform predicate crate option ...)")
    (summary
      "Create a package/platform override that installs a published Cargo crate.")
    (param 'predicate "Host predicate for this package rule.")
    (param 'crate "Crate package spec passed to `cargo install`.")
    (param
      'option
      "Additional cargo install flags or package platform fields. Field values are applied after defaults."))

  (define (cargo/crate-platform predicate-value crate . options)
    (call-with-split-fields
      options
      (lambda (flags fields)
        (object/merge
          (package/platform
            predicate-value
            (arr "cargo")
            (cargo/crate-install-argv "{{ package }}" (arr/append-list (arr) flags))
            (field 'name crate))
          fields))))

  (moduledoc
    (summary "Cargo helpers for Rust tools installed from source or published crates.")
    (group "Cargo")))
