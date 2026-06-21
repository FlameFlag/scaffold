(library
  (scaffold catalog archive)
  (export archive archive/strip-components dmg)
  (import (rnrs) (scaffold config) (scaffold catalog action))

  (doc-next
    (signature "(archive path field ...)")
    (summary "Create an action that extracts a local archive into the tool prefix.")
    (param 'path "Archive path relative to the Scaffold root.")
    (param 'field "Archive fields such as `strip-components`.")
    (returns
      "An action object with type `archive`. Supported formats are `.zip`, `.tar`, `.tar.gz`, `.tgz`, `.tar.bz2`, `.tbz2`, `.tar.xz`, `.txz`, and `.dmg`."))

  (define (archive path . fields)
    (cons* (field 'type 'archive) (field 'path path) fields))

  (doc-next
    (signature "(archive/strip-components path count field ...)")
    (summary
      "Create an archive action that removes leading path components while extracting.")
    (param 'path "Archive path relative to the Scaffold root.")
    (param 'count "Number of leading path components to remove from archive entries.")
    (param 'field "Additional archive fields."))

  (define (archive/strip-components path count . fields)
    (cons*
      (field 'type 'archive)
      (field 'path path)
      (field 'strip-components count)
      fields))

  (doc-next
    (signature "(dmg path field ...)")
    (summary "Create an action that mounts and extracts a local macOS DMG.")
    (param 'path "DMG path relative to the Scaffold root.")
    (param 'field "Additional archive fields.")
    (returns
      "An archive action for `.dmg` files. DMG extraction requires macOS `hdiutil` and `ditto`."))

  (define (dmg path . fields)
    (cons* (field 'type 'archive) (field 'path path) fields))

  (moduledoc (summary "Catalog archive extraction action helpers.") (group "Archives")))
