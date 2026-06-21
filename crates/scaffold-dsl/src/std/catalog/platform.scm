(library
  (scaffold catalog platform)
  (export predicate platform/package tool/path package/platform package/platform-argvs)
  (import (rnrs) (scaffold config))

  (doc-next
    (signature "(predicate os arch field ...)")
    (summary "Create an OS/architecture predicate for platform-specific catalog data."))

  (define (predicate os arch . fields)
    (cons* (field 'os os) (field 'arch arch) fields))

  (doc-next
    (signature "(platform/package predicate field ...)")
    (summary "Create a platform-specific package override."))

  (define (platform/package predicate-value . fields)
    (cons* (field 'when predicate-value) fields))

  (doc-next
    (summary "Describe a platform-specific path where a tool may already exist."))

  (define (tool/path predicate-value path)
    (object (field 'when predicate-value) (field 'path path)))

  (doc-next
    (signature "(package/platform predicate requires install-argv field ...)")
    (summary "Create a platform-specific package install rule with required commands."))

  (define (package/platform predicate-value requires install-argv . fields)
    (cons*
      (field 'when predicate-value)
      (field 'requires-commands requires)
      (field 'install-argv install-argv)
      fields))

  (doc-next
    (signature "(package/platform-argvs predicate requires install-argvs field ...)")
    (summary "Create a platform-specific package install rule with multiple commands.")
    (param 'predicate "Host predicate for this package rule.")
    (param 'requires "Commands that must be available for this platform rule.")
    (param 'install-argvs "Vector of install command argv vectors.")
    (param 'field "Additional platform fields."))

  (define (package/platform-argvs predicate-value requires install-argvs . fields)
    (cons*
      (field 'when predicate-value)
      (field 'requires-commands requires)
      (field 'install-argvs install-argvs)
      fields))

  (moduledoc
    (summary "Platform predicate and override constructors.")
    (group "Platforms")))
