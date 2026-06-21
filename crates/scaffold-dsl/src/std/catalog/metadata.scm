(library
  (scaffold catalog metadata)
  (export
    meta
    home-page
    description
    license
    maintainers
    tags
    main-program
    source
    passthru)
  (import (rnrs) (scaffold config))

  (doc-next
    (signature "(meta field ...)")
    (summary "Attach descriptive metadata to a catalog tool.")
    (param
      'field
      "Metadata fields such as `home-page`, `description`, `license`, `maintainers`, `tags`, `main-program`, or `source`.")
    (returns "A `meta` field accepted by `tool`."))

  (define (meta . fields) (field 'meta fields))

  (doc-next (summary "Create a metadata field for the tool's upstream homepage."))

  (define (home-page url) (field 'home-page url))

  (doc-next (summary "Create a longer metadata description for a tool."))

  (define (description text) (field 'description text))

  (doc-next (summary "Create a metadata field for a tool license identifier or note."))

  (define (license value) (field 'license value))

  (doc-next
    (signature "(maintainers name ...)")
    (summary "Create a metadata field listing maintainers for a catalog tool."))

  (define (maintainers . names) (field 'maintainers (list->arr names)))

  (doc-next
    (signature "(tags tag ...)")
    (summary "Create a metadata field listing searchable labels for a catalog tool."))

  (define (tags . values) (field 'tags (list->arr values)))

  (doc-next
    (summary "Create a metadata field naming the primary executable for a tool."))

  (define (main-program name) (field 'main-program name))

  (doc-next
    (summary "Create a metadata field describing the upstream source location."))

  (define (source value) (field 'source value))

  (doc-next
    (signature "(passthru field ...)")
    (summary "Attach open extension data to a catalog tool.")
    (param
      'field
      "Arbitrary object fields preserved in evaluated catalog JSON but ignored by installation."))

  (define (passthru . fields) (field 'passthru fields))

  (moduledoc
    (summary "Tool metadata and open extension-data helpers.")
    (group "Catalog")))
