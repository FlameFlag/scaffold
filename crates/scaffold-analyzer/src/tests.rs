use super::*;

#[test]
fn warns_for_missing_documentation() {
    let diagnostics = analyze_source("test.scm", "(define (helper x) x)");

    assert_eq!(diagnostics.len(), 1);
    assert!(!diagnostics[0].is_error());
    assert!(diagnostics[0].to_string().contains("helper"));
}

#[test]
fn reports_syntax_error() {
    let diagnostics = analyze_source("test.scm", "(define x");

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].is_error());
}

#[test]
fn reports_duplicate_definitions_with_an_error() {
    let diagnostics = analyze_source("test.scm", "(define value 1)\n(define value 2)");

    assert_eq!(diagnostics.len(), 3);
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.is_error() && diagnostic.to_string().contains("defined more than once")
    }));
}

#[test]
fn reports_duplicate_docs_with_a_warning() {
    let diagnostics = analyze_source(
        "test.scm",
        "(doc 'value (summary \"First.\"))\n(doc 'value (summary \"Second.\"))",
    );

    assert_eq!(diagnostics.len(), 1);
    assert!(!diagnostics[0].is_error());
    assert!(
        diagnostics[0]
            .to_string()
            .contains("more than one documentation entry")
    );
}

#[test]
fn reports_public_doc_entries_without_summary() {
    let diagnostics = analyze_source("test.scm", "(doc 'value (signature \"(value)\"))");

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].to_string().contains("missing a summary"));
}

#[test]
fn ignores_hidden_doc_entries_without_summary() {
    let diagnostics = analyze_source("test.scm", "(doc 'value (hidden))");

    assert!(diagnostics.is_empty());
}

#[test]
fn accepts_doc_next_entries() {
    let diagnostics = analyze_source(
        "test.scm",
        "(doc-next (summary \"Docs.\"))\n(define (helper x) x)",
    );

    assert!(diagnostics.is_empty());
}

#[test]
fn accepts_extern_doc_entries() {
    let diagnostics = analyze_source(
        "test.scm",
        "(extern-doc helper (summary \"Docs.\"))\n(define helper %helper)",
    );

    assert!(diagnostics.is_empty());
}

#[test]
fn accepts_docs_from_a_shared_index() {
    let mut docs = DocIndex::empty();
    docs.extend_source("facade.scm", "(doc 'helper (summary \"Documented.\"))");

    let diagnostics = analyze_source_with_docs(
        "impl.scm",
        "(library (impl) (export helper) (import (rnrs)) (define (helper) #t))",
        &docs,
    );

    assert!(diagnostics.is_empty());
}

#[test]
fn requires_docs_for_private_library_definitions() {
    let diagnostics = analyze_source(
        "test.scm",
        "(library (test) (export public) (import (rnrs)) (define private 1) (define public 2))",
    );

    assert_eq!(diagnostics.len(), 2);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.to_string().contains("private"))
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.to_string().contains("public"))
    );
}
