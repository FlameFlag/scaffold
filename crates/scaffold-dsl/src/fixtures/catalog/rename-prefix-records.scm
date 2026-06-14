(import
  (rnrs)
  (rename (scaffold catalog root) (catalog make-catalog))
  (rename (scaffold catalog action) (required action-required))
  (prefix (scaffold catalog tool) tool:)
  (only (scaffold config vector) arr)
  (only (scaffold config object) field))

(define-record-type tool-spec
  (fields name bin))

(define spec (make-tool-spec "demo" "democtl"))

(make-catalog
  (tool:tool
    (tool-spec-name spec)
    (action-required)
    (field 'bins (arr (tool:bin (tool-spec-bin spec))))))
