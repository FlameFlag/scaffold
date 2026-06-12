(library
  (scaffold extensions distro apt)
  (export
    apt-get/install-argv
    apt-get/remove-argv
    apt/package
    apt/package-tool
    apt/package-platform)
  (import (rnrs) (scaffold catalog base) (scaffold extensions support checks))

  (doc-next
    (signature "(apt-get/install-argv package ...)")
    (summary "Build argv for installing packages with `sudo apt-get install -y`.")
    (param 'package "APT package names.")
    (returns "Vector argv suitable for a package action `install-argv`."))

  (define (apt-get/install-argv . packages)
    (apply arr "sudo" "apt-get" "install" "-y" packages))

  (doc-next
    (signature "(apt-get/remove-argv package ...)")
    (summary "Build argv for removing packages with `sudo apt-get remove -y`.")
    (param 'package "APT package names.")
    (returns "Vector argv suitable for uninstall metadata."))

  (define (apt-get/remove-argv . packages)
    (apply arr "sudo" "apt-get" "remove" "-y" packages))

  (doc-next
    (signature "(apt/package name field ...)")
    (summary "Create a catalog tool installed through APT.")
    (param 'name "APT package name and default catalog tool name.")
    (param 'field "Additional tool fields that override defaults.")
    (returns "A tool with an APT package action and `dpkg-query -W` check."))

  (define (apt/package name . fields)
    (apply
      tool
      name
      (package
        (field 'name name)
        (field 'install-argv (apt-get/install-argv "{{ package }}")))
      (field 'platforms (arr 'linux))
      (field 'checks (arr (command/check "dpkg-query" "-W" "{{ package }}")))
      (field
        'uninstall
        (uninstall
          (field
            'commands
            (arr (host/uninstall-command 'linux (apt-get/remove-argv "{{ package }}"))))))
      fields))

  (doc-next
    (signature "(apt/package-tool name package-name bin-name field ...)")
    (summary
      "Create a catalog tool installed through an APT package with a separate tool name.")
    (param 'name "Catalog tool name.")
    (param 'package-name "APT package name.")
    (param 'bin-name "Executable exposed by the package.")
    (param 'field "Additional tool fields that override defaults."))

  (define (apt/package-tool name package-name bin-name . fields)
    (apply
      tool
      name
      (package
        (field 'name package-name)
        (field 'install-argv (apt-get/install-argv "{{ package }}")))
      (field 'platforms (arr 'linux))
      (field 'checks (arr (command/check "dpkg-query" "-W" "{{ package }}")))
      (field 'bins (arr (bin bin-name)))
      (field
        'uninstall
        (uninstall
          (field
            'commands
            (arr (host/uninstall-command 'linux (apt-get/remove-argv "{{ package }}"))))))
      fields))

  (doc-next
    (signature "(apt/package-platform package-name field ...)")
    (summary "Create a Linux package/platform override that requires `apt-get`.")
    (param 'package-name "APT package name.")
    (param 'field "Additional platform fields that override defaults."))

  (define (apt/package-platform package-name . fields)
    (apply
      package/platform
      'linux
      (arr "apt-get")
      (apt-get/install-argv "{{ package }}")
      (field 'name package-name)
      fields))

  (moduledoc
    (summary "APT package helpers for Debian and Ubuntu style systems.")
    (group "Distro packages")))
