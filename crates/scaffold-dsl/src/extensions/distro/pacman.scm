(library
  (scaffold extensions distro pacman)
  (export
    pacman/install-argv
    pacman/remove-argv
    pacman/package
    pacman/package-tool
    pacman/package-platform)
  (import (rnrs) (scaffold catalog base) (scaffold extensions support checks))

  (doc-next
    (signature "(pacman/install-argv package ...)")
    (summary
      "Build argv for installing packages with `sudo pacman -S --needed --noconfirm`.")
    (param 'package "Pacman package names.")
    (returns "Vector argv suitable for a package action `install-argv`."))

  (define (pacman/install-argv . packages)
    (arr/append-list (arr "sudo" "pacman" "-S" "--needed" "--noconfirm") packages))

  (doc-next
    (signature "(pacman/remove-argv package ...)")
    (summary "Build argv for removing packages with `sudo pacman -R --noconfirm`.")
    (param 'package "Pacman package names.")
    (returns "Vector argv suitable for uninstall metadata."))

  (define (pacman/remove-argv . packages)
    (arr/append-list (arr "sudo" "pacman" "-R" "--noconfirm") packages))

  (doc-next
    (signature "(pacman/package name field ...)")
    (summary "Create a catalog tool installed through Pacman.")
    (param 'name "Pacman package name and default catalog tool name.")
    (param 'field "Additional tool fields that override defaults.")
    (returns "A tool with a Pacman package action and `pacman -Q` check."))

  (define (pacman/package name . fields)
    (object/merge
      (tool
        name
        (package
          (field 'name name)
          (field 'install-argv (pacman/install-argv "{{ package }}")))
        (field 'platforms (arr 'linux))
        (field 'checks (arr (command/check "pacman" "-Q" "{{ package }}")))
        (field
          'uninstall
          (uninstall
            (field
              'commands
              (arr
                (host/uninstall-command 'linux (pacman/remove-argv "{{ package }}")))))))
      fields))

  (doc-next
    (signature "(pacman/package-tool name package-name bin-name field ...)")
    (summary
      "Create a catalog tool installed through a Pacman package with a separate tool name.")
    (param 'name "Catalog tool name.")
    (param 'package-name "Pacman package name.")
    (param 'bin-name "Executable exposed by the package.")
    (param 'field "Additional tool fields that override defaults."))

  (define (pacman/package-tool name package-name bin-name . fields)
    (object/merge
      (tool
        name
        (package
          (field 'name package-name)
          (field 'install-argv (pacman/install-argv "{{ package }}")))
        (field 'platforms (arr 'linux))
        (field 'checks (arr (command/check "pacman" "-Q" "{{ package }}")))
        (field 'bins (arr (bin bin-name)))
        (field
          'uninstall
          (uninstall
            (field
              'commands
              (arr
                (host/uninstall-command 'linux (pacman/remove-argv "{{ package }}")))))))
      fields))

  (doc-next
    (signature "(pacman/package-platform package-name field ...)")
    (summary "Create a Linux package/platform override that requires `pacman`.")
    (param 'package-name "Pacman package name.")
    (param 'field "Additional platform fields that override defaults."))

  (define (pacman/package-platform package-name . fields)
    (object/merge
      (package/platform
        'linux
        (arr "pacman")
        (pacman/install-argv "{{ package }}")
        (field 'name package-name))
      fields))

  (moduledoc
    (summary "Pacman package helpers for Arch style systems.")
    (group "Distro packages")))
