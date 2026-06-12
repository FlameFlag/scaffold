(library
  (scaffold extensions distro dnf)
  (export
    dnf/install-argv
    dnf/remove-argv
    dnf/package
    dnf/package-tool
    dnf/package-platform)
  (import (rnrs) (scaffold catalog base) (scaffold extensions support checks))

  (doc-next
    (signature "(dnf/install-argv package ...)")
    (summary "Build argv for installing packages with `sudo dnf install -y`.")
    (param 'package "DNF package names.")
    (returns "Vector argv suitable for a package action `install-argv`."))

  (define (dnf/install-argv . packages)
    (apply arr "sudo" "dnf" "install" "-y" packages))

  (doc-next
    (signature "(dnf/remove-argv package ...)")
    (summary "Build argv for removing packages with `sudo dnf remove -y`.")
    (param 'package "DNF package names.")
    (returns "Vector argv suitable for uninstall metadata."))

  (define (dnf/remove-argv . packages) (apply arr "sudo" "dnf" "remove" "-y" packages))

  (doc-next
    (signature "(dnf/package name field ...)")
    (summary "Create a catalog tool installed through DNF.")
    (param 'name "DNF package name and default catalog tool name.")
    (param 'field "Additional tool fields that override defaults.")
    (returns "A tool with a DNF package action and `rpm -q` check."))

  (define (dnf/package name . fields)
    (apply
      tool
      name
      (package
        (field 'name name)
        (field 'install-argv (dnf/install-argv "{{ package }}")))
      (field 'platforms (arr 'linux))
      (field 'checks (arr (command/check "rpm" "-q" "{{ package }}")))
      (field
        'uninstall
        (uninstall
          (field
            'commands
            (arr (host/uninstall-command 'linux (dnf/remove-argv "{{ package }}"))))))
      fields))

  (doc-next
    (signature "(dnf/package-tool name package-name bin-name field ...)")
    (summary
      "Create a catalog tool installed through a DNF package with a separate tool name.")
    (param 'name "Catalog tool name.")
    (param 'package-name "DNF package name.")
    (param 'bin-name "Executable exposed by the package.")
    (param 'field "Additional tool fields that override defaults."))

  (define (dnf/package-tool name package-name bin-name . fields)
    (apply
      tool
      name
      (package
        (field 'name package-name)
        (field 'install-argv (dnf/install-argv "{{ package }}")))
      (field 'platforms (arr 'linux))
      (field 'checks (arr (command/check "rpm" "-q" "{{ package }}")))
      (field 'bins (arr (bin bin-name)))
      (field
        'uninstall
        (uninstall
          (field
            'commands
            (arr (host/uninstall-command 'linux (dnf/remove-argv "{{ package }}"))))))
      fields))

  (doc-next
    (signature "(dnf/package-platform package-name field ...)")
    (summary "Create a Linux package/platform override that requires `dnf`.")
    (param 'package-name "DNF package name.")
    (param 'field "Additional platform fields that override defaults."))

  (define (dnf/package-platform package-name . fields)
    (apply
      package/platform
      'linux
      (arr "dnf")
      (dnf/install-argv "{{ package }}")
      (field 'name package-name)
      fields))

  (moduledoc
    (summary "DNF package helpers for Fedora style systems.")
    (group "Distro packages")))
