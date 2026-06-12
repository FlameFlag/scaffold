(library
  (scaffold core doc)
  (export
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
    hidden)
  (import (rnrs) (scaffold core object))

  (define (doc-field name value) (field name value))

  (define (signature text) (doc-field 'signature text))

  (define (summary text) (doc-field 'summary text))

  (define (markdown text) (doc-field 'markdown text))

  (define (example text) (doc-field 'example text))

  (define (param name text)
    (doc-field 'param (object (field 'name name) (field 'summary text))))

  (define (returns text) (doc-field 'returns text))

  (define (group text) (doc-field 'group text))

  (define (see subject) (doc-field 'see subject))

  (define (effect name) (doc-field 'effect name))

  (define (requires-capability name) (doc-field 'requires-capability name))

  (define (stability text) (doc-field 'stability text))

  (define (since text) (doc-field 'since text))

  (define (deprecated text) (doc-field 'deprecated text))

  (define (hidden) (doc-field 'hidden #t))

  (define (doc subject . fields)
    (apply
      object
      (field 'scaffold:kind "doc")
      (field 'doc:kind "value")
      (field 'subject subject)
      fields))

  (define (doc-next . fields)
    (apply object (field 'scaffold:kind "doc") (field 'doc:kind "next") fields))

  (define (extern-doc subject . fields)
    (apply
      object
      (field 'scaffold:kind "doc")
      (field 'doc:kind "extern")
      (field 'subject subject)
      fields))

  (define (moduledoc . fields)
    (apply object (field 'scaffold:kind "doc") (field 'doc:kind "module") fields))

  (define (typedoc subject . fields)
    (apply
      object
      (field 'scaffold:kind "doc")
      (field 'doc:kind "type")
      (field 'subject subject)
      fields)))
