use super::source::docs_from_source;
use super::*;

#[test]
fn extracts_doc_forms_from_scheme_source() {
    let docs = docs_from_source("doc-entry.scm", include_str!("../fixtures/doc-entry.scm"));

    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].name, "demo");
    assert_eq!(docs[0].signature.as_deref(), Some("(demo name)"));
    assert_eq!(docs[0].summary.as_deref(), Some("Create a demo."));
    assert_eq!(docs[0].markdown.as_deref(), Some("More docs."));
    assert_eq!(docs[0].params[0].name, "name");
    assert_eq!(docs[0].params[0].summary, "Name for the demo.");
    assert_eq!(docs[0].returns.as_deref(), Some("A demo object."));
    assert_eq!(docs[0].group.as_deref(), Some("Fixtures"));
    assert_eq!(docs[0].see, vec!["other-demo".to_owned()]);
    assert!(docs[0].range.is_some());
}

#[test]
fn extracts_doc_next_from_following_definition() {
    let docs = docs_from_source(
        "doc-next.scm",
        r#"(moduledoc (summary "Module docs.") (group "Fixtures") (effect 'pure))
(doc-next
  (summary "Create a demo.")
  (param 'name "Name for the demo."))
(define (demo name) name)"#,
    );

    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].name, "demo");
    assert_eq!(docs[0].signature.as_deref(), Some("(demo name)"));
    assert_eq!(docs[0].summary.as_deref(), Some("Create a demo."));
    assert_eq!(docs[0].group.as_deref(), Some("Fixtures"));
    assert_eq!(docs[0].effect.as_deref(), Some("pure"));
    assert_eq!(docs[0].params[0].name, "name");
}

#[test]
fn extracts_extern_doc_and_structured_signature() {
    let docs = docs_from_source(
        "extern-doc.scm",
        r#"(moduledoc (summary "Path docs.") (group "Paths") (requires-capability 'scaffold.path))
(extern-doc path/join
  (signature (path/join first part ...))
  (summary "Join path components."))
(define path/join %path/join)
(extern-doc path/separator
  (signature value path/separator)
  (summary "Host path separator."))
(define path/separator %path/separator)"#,
    );

    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0].name, "path/join");
    assert_eq!(
        docs[0].signature.as_deref(),
        Some("(path/join first part ...)")
    );
    assert_eq!(
        docs[0].requires_capability,
        vec!["scaffold.path".to_owned()]
    );
    assert_eq!(docs[1].name, "path/separator");
    assert_eq!(docs[1].signature.as_deref(), Some("path/separator"));
}

#[test]
fn extracts_doc_forms_from_sources_with_scaffold_keywords() {
    let docs = docs_from_source(
        "keyword-doc.scm",
        r#"(tool #:name "demo")
(doc 'local-helper
  (signature "(local-helper value)")
  (summary "Project-local docs.")
  (param 'value "Input value."))
(define (local-helper value) value)"#,
    );

    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].name, "local-helper");
    assert_eq!(docs[0].signature.as_deref(), Some("(local-helper value)"));
    assert_eq!(docs[0].summary.as_deref(), Some("Project-local docs."));
    assert_eq!(docs[0].params[0].name, "value");
    assert_eq!(docs[0].params[0].summary, "Input value.");
    assert_eq!(docs[0].range.expect("definition range").start.line, 5);
}

#[test]
fn editor_docs_index_undocumented_definitions_as_symbols() {
    let docs = source_docs_with_definitions(
        "local.scm",
        "(define (local-helper value) value)\n(local-helper 1)",
    )
    .entries;

    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].name, "local-helper");
    assert_eq!(docs[0].source.as_deref(), Some("local.scm"));
    assert_eq!(docs[0].range.expect("definition range").start.line, 0);
    assert!(docs[0].summary.is_none());
}

#[test]
fn editor_docs_keep_definition_location_when_docs_exist() {
    let docs = source_docs_with_definitions(
        "keyword-doc.scm",
        r#"(tool #:name "demo")
(doc 'local-helper
  (signature "(local-helper value)")
  (summary "Project-local docs."))
(define (local-helper value) value)"#,
    )
    .entries;

    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].summary.as_deref(), Some("Project-local docs."));
    assert_eq!(docs[0].range.expect("definition range").start.line, 4);
}

#[cfg(feature = "reference")]
#[test]
fn scaffold_index_reads_std_docs() {
    let index = DocIndex::scaffold();

    let tool = index.get("tool").expect("tool doc");
    assert!(
        tool.signature
            .as_deref()
            .is_some_and(|sig| sig.contains("tool"))
    );
    assert!(index.get("arr").is_some());
    assert!(index.get("path/exists?").is_some());
    assert!(index.get("path/join").is_some());
    assert!(index.get("command/path").is_some());
    assert!(index.get("host/matches?").is_some());
    assert!(index.get("workspace/path").is_some());
    assert!(index.get("nix/profile-package").is_some());
}

#[test]
fn document_docs_do_not_override_language_keywords() {
    let index = DocIndex::with_language_keywords()
        .merged_with_document("open.scm", include_str!("../fixtures/override-doc.scm"));

    assert_eq!(
        index
            .get("define")
            .and_then(|entry| entry.summary.as_deref()),
        Some("Bind a Scheme value or procedure in the current scope.")
    );
}

#[cfg(feature = "reference")]
#[test]
fn document_docs_override_non_keyword_entries() {
    let index = DocIndex::scaffold().merged_with_document(
        "open.scm",
        "(doc 'tool (summary \"Project-local tool docs.\"))",
    );

    assert_eq!(
        index.get("tool").and_then(|entry| entry.summary.as_deref()),
        Some("Project-local tool docs.")
    );
}

#[test]
fn builds_snippet_from_signature() {
    assert_eq!(
        snippet_for_signature("(tool name action field ...)").as_deref(),
        Some("(tool ${1:name} ${2:action} ${3:field} ...)")
    );
}

#[test]
fn markdown_includes_structured_fields() {
    let docs = docs_from_source("doc-entry.scm", include_str!("../fixtures/doc-entry.scm"));
    let markdown = markdown_for_entry(&docs[0]);

    assert!(markdown.contains("Group: Fixtures"));
    assert!(markdown.contains("Parameters:"));
    assert!(markdown.contains("Returns: A demo object."));
    assert!(markdown.contains("See also: `other-demo`"));
}

#[test]
fn filters_entries_by_source() {
    let index = DocIndex::with_language_keywords()
        .merged_with_document("doc-entry.scm", include_str!("../fixtures/doc-entry.scm"));
    let entries = index
        .entries_in_source("doc-entry.scm")
        .map(|entry| entry.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(entries, vec!["demo"]);
}
