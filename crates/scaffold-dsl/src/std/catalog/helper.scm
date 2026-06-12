(library
  (scaffold catalog helper)
  (export
    tool/prefer-present
    workspace/build
    tool/platforms
    host/package-platform
    host/package-platform-argvs
    uninstall/paths)
  (import
    (rnrs)
    (scaffold config)
    (scaffold host)
    (scaffold workspace)
    (scaffold catalog action)
    (scaffold catalog platform)
    (scaffold catalog tool)
    (scaffold catalog uninstall))

  (doc-next
    (signature "(tool/prefer-present name package-action field ...)")
    (summary
      "Create a required tool when the command is already present, otherwise use a package action.")
    (param 'name "Command and tool name to prefer from the host when available.")
    (param 'package-action "Package action used when the command is not available.")
    (returns "A tool object with either a `required` or package action."))

  (define (tool/prefer-present name package-action . fields)
    (apply tool name (if (command/available? name) (required) package-action) fields))

  (doc-next
    (signature "(workspace/build path field ...)")
    (summary
      "Create a build action whose source path is resolved from the workspace root.")
    (param 'path "Path relative to `workspace/root`.")
    (param 'field "Build fields such as `argv` or ordered `argvs`."))

  (define (workspace/build path . fields)
    (apply build (field 'path (workspace/path path)) fields))

  (doc-next
    (signature "(tool/platforms platform ...)")
    (summary "Create a `platforms` field for a tool.")
    (param 'platform "Host OS symbol or string such as `linux`, `macos`, or `windows`."))

  (define (tool/platforms . platforms) (field 'platforms (list->vector platforms)))

  (doc-next
    (signature "(host/package-platform requires install-argv field ...)")
    (summary "Create a package platform override for the current host OS.")
    (param 'requires "Commands that must be available for this platform rule.")
    (param 'install-argv "Install command argv vector."))

  (define (host/package-platform requires install-argv . fields)
    (apply package/platform host/os requires install-argv fields))

  (doc-next
    (signature "(host/package-platform-argvs requires install-argvs field ...)")
    (summary
      "Create a package platform override with multiple commands for the current host OS.")
    (param 'requires "Commands that must be available for this platform rule.")
    (param 'install-argvs "Vector of install command argv vectors.")
    (param 'field "Additional platform fields."))

  (define (host/package-platform-argvs requires install-argvs . fields)
    (apply package/platform-argvs host/os requires install-argvs fields))

  (doc-next
    (signature "(uninstall/paths path ...)")
    (summary "Create an `uninstall` field containing path removal entries.")
    (param 'path "Path template removed by `scaffold uninstall`."))

  (define (uninstall/paths . paths)
    (field
      'uninstall
      (uninstall (field 'paths (list->vector (map uninstall/path paths))))))

  (moduledoc (summary "Catalog domain convenience helpers.") (group "Catalog")))
