(library
  (scaffold extensions distro rpm)
  (export
    rpm-ostree/install-argv
    rpm-ostree/uninstall-argv
    rpm-ostree/package
    rpm-ostree/package-tool
    rpm-ostree/package-platform
    rpm-ostree/packages-platform)
  (import (rnrs) (scaffold catalog base) (scaffold extensions support checks))

  (doc-next
    (signature "(rpm-ostree/install-argv package ...)")
    (summary
      "Build argv for installing packages with `sudo rpm-ostree install --idempotent -y`.")
    (param 'package "RPM package names.")
    (returns "Vector argv suitable for a package action `install-argv`."))

  (doc-next
    (hidden)
    (summary "Build rpm-ostree install argv from an existing package list."))

  (define (rpm-ostree/install-argv-list packages)
    (arr/append-list
      (arr "sudo" "rpm-ostree" "install" "--idempotent" "-y")
      packages))

  (define (rpm-ostree/install-argv . packages)
    (rpm-ostree/install-argv-list packages))

  (doc-next
    (signature "(rpm-ostree/uninstall-argv package ...)")
    (summary
      "Build argv for uninstalling packages with `sudo rpm-ostree uninstall -y`.")
    (param 'package "RPM package names.")
    (returns "Vector argv suitable for uninstall metadata."))

  (define (rpm-ostree/uninstall-argv . packages)
    (arr/append-list (arr "sudo" "rpm-ostree" "uninstall" "-y") packages))

  (doc-next
    (signature "(rpm-ostree/package name field ...)")
    (summary "Create a catalog tool installed through rpm-ostree.")
    (param 'name "RPM package name and default catalog tool name.")
    (param 'field "Additional tool fields that override defaults.")
    (returns "A tool with an rpm-ostree package action and `rpm -q` check."))

  (define (rpm-ostree/package name . fields)
    (object/merge
      (tool
        name
        (package
          (field 'name name)
          (field 'install-argv (rpm-ostree/install-argv "{{ package }}")))
        (field 'platforms (arr 'linux))
        (field 'checks (arr (command/check "rpm" "-q" "{{ package }}")))
        (field
          'uninstall
          (uninstall
            (field
              'commands
              (arr
                (host/uninstall-command
                  'linux
                  (rpm-ostree/uninstall-argv "{{ package }}")))))))
      fields))

  (doc-next
    (signature "(rpm-ostree/package-tool name package-name bin-name field ...)")
    (summary
      "Create a catalog tool installed through an rpm-ostree package with a separate tool name.")
    (param 'name "Catalog tool name.")
    (param 'package-name "RPM package name.")
    (param 'bin-name "Executable exposed by the package.")
    (param 'field "Additional tool fields that override defaults."))

  (define (rpm-ostree/package-tool name package-name bin-name . fields)
    (object/merge
      (tool
        name
        (package
          (field 'name package-name)
          (field 'install-argv (rpm-ostree/install-argv "{{ package }}")))
        (field 'platforms (arr 'linux))
        (field 'checks (arr (command/check "rpm" "-q" "{{ package }}")))
        (field 'bins (arr (bin bin-name)))
        (field
          'uninstall
          (uninstall
            (field
              'commands
              (arr
                (host/uninstall-command
                  'linux
                  (rpm-ostree/uninstall-argv "{{ package }}")))))))
      fields))

  (doc-next
    (signature "(rpm-ostree/package-platform package-name field ...)")
    (summary "Create a Linux package/platform override that requires `rpm-ostree`.")
    (param 'package-name "RPM package name.")
    (param 'field "Additional platform fields that override defaults."))

  (define (rpm-ostree/package-platform package-name . fields)
    (object/merge
      (package/platform
        'linux
        (arr "rpm-ostree")
        (rpm-ostree/install-argv "{{ package }}")
        (field 'name package-name))
      fields))

  (doc-next
    (signature "(rpm-ostree/packages-platform name package-or-field ...)")
    (summary "Create a named Linux package/platform override for one or more rpm-ostree packages.")
    (param 'name "Platform rule name.")
    (param 'package-or-field "RPM package names followed by optional platform fields."))

  (define (rpm-ostree/packages-platform name . options)
    (call-with-split-fields
      options
      (lambda (packages fields)
        (object/merge
          (package/platform
            'linux
            (arr "rpm-ostree")
            (rpm-ostree/install-argv-list packages)
            (field 'name name))
          fields))))

  (moduledoc
    (summary "rpm-ostree package helpers for image-based Fedora systems.")
    (group "Distro packages")))
