(library
  (scaffold extensions app flatpak)
  (export
    flatpak/install-argv
    flatpak/uninstall-argv
    flatpak/app
    flatpak/package-platform)
  (import (rnrs) (scaffold catalog base) (scaffold extensions support checks))

  (doc-next
    (summary "Build argv for installing a Flatpak app non-interactively.")
    (param 'remote "Flatpak remote, usually `flathub`.")
    (param 'app-id "Flatpak application ID.")
    (returns "Vector argv for `flatpak install --assumeyes --noninteractive`."))

  (define (flatpak/install-argv remote app-id)
    (arr "flatpak" "install" "--assumeyes" "--noninteractive" remote app-id))

  (doc-next
    (summary "Build argv for uninstalling a Flatpak app non-interactively.")
    (param 'app-id "Flatpak application ID.")
    (returns "Vector argv for `flatpak uninstall --assumeyes --noninteractive`."))

  (define (flatpak/uninstall-argv app-id)
    (arr "flatpak" "uninstall" "--assumeyes" "--noninteractive" app-id))

  (doc-next
    (signature "(flatpak/app name app-id field ...)")
    (summary "Create a Linux-only catalog tool installed from Flathub.")
    (param 'name "Catalog tool name.")
    (param 'app-id "Flatpak application ID used for install, checks, and version argv.")
    (param 'field "Additional tool fields that override defaults.")
    (returns
      "A tool with package action, Flatpak info check, binary metadata, and Linux platform tag."))

  (define (flatpak/app name app-id . fields)
    (object/merge
      (tool
        name
        (package
          (field 'name app-id)
          (field 'install-argv (flatpak/install-argv "flathub" "{{ package }}")))
        (field 'platforms (arr 'linux))
        (field 'checks (arr (command/check "flatpak" "info" app-id)))
        (field
          'bins
          (arr (bin app-id (field 'version-argv (arr "flatpak" "info" app-id)))))
        (field
          'uninstall
          (uninstall
            (field
              'commands
              (arr
                (host/uninstall-command
                  'linux
                  (flatpak/uninstall-argv "{{ package }}")))))))
      fields))

  (doc-next
    (signature "(flatpak/package-platform app-id field ...)")
    (summary "Create a Linux package/platform override that requires Flatpak.")
    (param 'app-id "Flatpak application ID installed from Flathub.")
    (param 'field "Additional platform fields that override defaults."))

  (define (flatpak/package-platform app-id . fields)
    (object/merge
      (package/platform
        'linux
        (arr "flatpak")
        (flatpak/install-argv "flathub" "{{ package }}")
        (field 'name app-id))
      fields))

  (moduledoc (summary "Flatpak application catalog helpers.") (group "Applications")))
