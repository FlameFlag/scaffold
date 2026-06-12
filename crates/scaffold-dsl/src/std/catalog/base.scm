(library
  (scaffold catalog base)
  (export
    field
    field/name
    field/value
    object
    arr
    vector/map
    vector/append
    arr/append-list
    arr/prepend-list
    object/has-field?
    object/ref
    object/remove-fields
    object/replace-fields
    object/merge
    object/inherit
    object/override
    object/replace-field
    object/append-field-vector
    object/append-vector
    doc
    doc-next
    extern-doc
    moduledoc
    typedoc
    signature
    summary
    markdown
    example
    param
    returns
    group
    see
    effect
    requires-capability
    stability
    since
    deprecated
    hidden
    catalog
    catalog/tool
    required
    required/tool
    package
    package/map-install-argvs
    build
    archive
    archive/strip-components
    dmg
    meta
    home-page
    description
    license
    maintainers
    tags
    main-program
    source
    passthru
    install/order
    depends
    install/before
    install/after
    uninstall
    uninstall/command
    host/uninstall-command
    uninstall/path
    host/uninstall-path
    tool
    tool/inherit
    tool/override
    tool/map-package-install-argvs
    tool/map-check-argvs
    uninstall/map-command-argvs
    tool/map-uninstall-command-argvs
    tool/append-bins
    install-argv/prepend
    argv/append
    tool/prefer-present
    workspace/build
    tool/platforms
    host/package-platform
    host/package-platform-argvs
    uninstall/paths
    predicate
    platform/package
    check
    host/check
    bin
    bin/version
    tool/path
    package/platform
    package/platform-argvs)
  (import
    (rnrs)
    (scaffold config)
    (scaffold catalog root)
    (scaffold catalog action)
    (scaffold catalog dependency)
    (scaffold catalog uninstall)
    (rename (scaffold catalog tool) (inherit tool/inherit) (override tool/override))
    (scaffold catalog archive)
    (scaffold catalog metadata)
    (scaffold catalog platform)
    (scaffold catalog check)
    (scaffold catalog transform)
    (scaffold catalog helper))

  (moduledoc
    (summary
      "Public catalog facade over focused action, tool, check, platform, dependency, uninstall, and transform modules.")
    (group "Catalog"))

  (extern-doc tool/inherit
    (signature "(tool/inherit base field ...)")
    (summary
      "Facade name for deriving a tool object by replacing fields on an existing tool.")
    (see 'inherit))

  (extern-doc tool/override
    (signature "(tool/override base proc)")
    (summary
      "Facade name for deriving a tool from overrides computed from the existing tool object.")
    (param 'base "Tool object to derive from.")
    (param
      'proc
      "Procedure that receives `base` and returns an object of replacement fields.")
    (see 'override)))
