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
    let docs = docs_from_source("doc-next.scm", include_str!("../fixtures/doc-next.scm"));

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
    let docs = docs_from_source("extern-doc.scm", include_str!("../fixtures/extern-doc.scm"));

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
        include_str!("../fixtures/keyword-doc.scm"),
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
        include_str!("../fixtures/keyword-doc-location.scm"),
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

    assert!(markdown.contains("| Field | Value    |"));
    assert!(markdown.contains("| Group | Fixtures |"));
    assert!(markdown.contains("**Parameters**"));
    assert!(markdown.contains("| Parameter | Description"));
    assert!(markdown.contains("**Returns:** A demo object."));
    assert!(markdown.contains("**See also:** `other-demo`"));
}

#[test]
fn entry_documentation_trims_deduplicates_and_builds_details() {
    let mut entry = DocEntry::new("demo", DocKind::Function);
    entry.signature = Some("  (demo value)  ".to_owned());
    entry.summary = Some("  Create a demo value.  ".to_owned());
    entry.markdown = Some("\nCreate a demo value.\n".to_owned());
    entry.returns = Some("  A demo object.  ".to_owned());
    entry.group = Some("Fixtures".to_owned());
    entry.requires_capability = vec!["scaffold.one".to_owned(), "scaffold.two".to_owned()];
    entry.source = Some("fixture.scm".to_owned());
    entry.range = Some(SourceRange {
        start: SourcePosition {
            line: 2,
            character: 4,
        },
        end: SourcePosition {
            line: 2,
            character: 8,
        },
    });

    let documentation = entry_documentation(&entry);

    assert!(documentation.has_body());
    assert_eq!(documentation.signature, Some("(demo value)"));
    assert_eq!(documentation.summary, Some("Create a demo value."));
    assert_eq!(documentation.markdown, None);
    assert_eq!(documentation.returns, Some("A demo object."));
    assert!(documentation.details.iter().any(|detail| {
        detail.field == "source"
            && detail.value == "fixture.scm:3"
            && detail.markdown_value == "`fixture.scm:3`"
    }));
    assert!(documentation.details.iter().any(|detail| {
        detail.field == "requires capability"
            && detail.value == "scaffold.one, scaffold.two"
            && detail.markdown_value == "`scaffold.one`, `scaffold.two`"
    }));
}

#[test]
fn group_markdown_table_formats_shared_group_rows() {
    let markdown = group_markdown_table([("Catalog", 35), ("Language", 12)]);

    assert!(markdown.contains("| Group"));
    assert!(markdown.contains("| Entries"));
    assert!(markdown.contains("`Catalog`"));
    assert!(markdown.contains("| 35"));
    assert!(markdown.contains("`Language`"));
}

#[test]
fn entry_count_labels_pluralize() {
    assert_eq!(entry_count_label(1), "1 entry");
    assert_eq!(entry_count_label(2), "2 entries");
    assert_eq!(group_count_label(1), "1 group");
    assert_eq!(group_count_label(2), "2 groups");
}

#[test]
fn entry_summary_markdown_table_formats_shared_entry_rows() {
    let mut entry = DocEntry::new("demo", DocKind::Function);
    entry.summary = Some("Create a demo value.".to_owned());
    entry.group = Some("Fixtures".to_owned());

    let markdown = entry_summary_markdown_table([&entry]);

    assert!(markdown.contains("| Symbol"));
    assert!(markdown.contains("| Group"));
    assert!(markdown.contains("| Summary"));
    assert!(markdown.contains("`demo`"));
    assert!(markdown.contains("Fixtures"));
    assert!(markdown.contains("Create a demo value."));
}

#[test]
fn entry_summary_markdown_table_escapes_group_markup() {
    let mut entry = DocEntry::new("demo", DocKind::Function);
    entry.summary = Some("Create a demo value.".to_owned());
    entry.group = Some("Bad [Group] | Plus+".to_owned());

    let markdown = entry_summary_markdown_table([&entry]);

    assert!(markdown.contains("Bad \\[Group\\] \\| Plus\\+"));
    assert!(!markdown.contains("Bad [Group] | Plus+"));
}

#[test]
fn detailed_markdown_adds_heading_body_details_and_see_also() {
    let mut entry = DocEntry::new("demo", DocKind::Function);
    entry.signature = Some("(demo value)".to_owned());
    entry.summary = Some("Create a demo value.".to_owned());
    entry.markdown = Some("Create a demo value.".to_owned());
    entry.params.push(DocParam {
        name: "value".to_owned(),
        summary: "Value to wrap.".to_owned(),
    });
    entry.returns = Some("A demo object.".to_owned());
    entry.example = Some("(demo 'ok)".to_owned());
    entry.group = Some("Fixtures".to_owned());
    entry.effect = Some("pure".to_owned());
    entry.requires_capability = vec!["scaffold.fixture".to_owned()];
    entry.see = vec!["other-demo".to_owned()];
    entry.source = Some("fixture.scm".to_owned());
    entry.range = Some(SourceRange {
        start: SourcePosition {
            line: 2,
            character: 4,
        },
        end: SourcePosition {
            line: 2,
            character: 8,
        },
    });

    let markdown = detailed_markdown_for_entry(&entry);

    assert!(markdown.starts_with("## `demo`"));
    assert!(markdown.contains("```scheme\n(demo value)\n```"));
    assert_eq!(markdown.matches("Create a demo value.").count(), 1);
    assert!(markdown.contains("### Parameters"));
    assert!(markdown.contains("| Parameter"));
    assert!(markdown.contains("`value`"));
    assert!(markdown.contains("### Returns"));
    assert!(markdown.contains("A demo object."));
    assert!(markdown.contains("### Example"));
    assert!(markdown.contains("(demo 'ok)"));
    assert!(markdown.contains("### Details"));
    assert!(markdown.contains("`Fixtures`"));
    assert!(markdown.contains("`fixture.scm:3`"));
    assert!(markdown.contains("`pure`"));
    assert!(markdown.contains("`scaffold.fixture`"));
    assert!(markdown.contains("### See also"));
    assert!(markdown.contains("`other-demo`"));
}

#[test]
fn detailed_markdown_adds_no_documentation_fallback() {
    let entry = DocEntry::new("sparse", DocKind::Function);

    let markdown = detailed_markdown_for_entry(&entry);

    assert!(markdown.starts_with("## `sparse`"));
    assert!(markdown.contains("No documentation provided."));
    assert!(markdown.contains("### Details"));
    assert!(markdown.contains("`Language`"));
}

#[test]
fn titled_markdown_adds_heading_fallback_and_source() {
    let mut entry = DocEntry::new("sparse", DocKind::Function);
    entry.source = Some("fixture.scm".to_owned());
    entry.range = Some(SourceRange {
        start: SourcePosition {
            line: 2,
            character: 4,
        },
        end: SourcePosition {
            line: 2,
            character: 10,
        },
    });

    let markdown = titled_markdown_for_entry(&entry);

    assert!(markdown.starts_with("## `sparse`"));
    assert!(markdown.contains("No documentation provided."));
    assert!(markdown.contains("| Source"));
    assert!(markdown.contains("`fixture.scm:3`"));
}

#[test]
fn source_markdown_adds_title_source_and_signature() {
    let mut entry = DocEntry::new("demo", DocKind::Function);
    entry.signature = Some("(demo value)".to_owned());
    entry.source = Some("fixture.scm".to_owned());
    entry.range = Some(SourceRange {
        start: SourcePosition {
            line: 2,
            character: 4,
        },
        end: SourcePosition {
            line: 2,
            character: 8,
        },
    });

    let markdown = source_markdown_for_entry(&entry).expect("source markdown");

    assert!(markdown.starts_with("## `demo` source"));
    assert!(markdown.contains("| Source"));
    assert!(markdown.contains("`fixture.scm:3`"));
    assert!(markdown.contains("| Signature"));
    assert!(markdown.contains("`(demo value)`"));
}

#[test]
fn markdown_renderers_escape_code_span_values_with_backticks() {
    let mut entry = DocEntry::new("bad`name", DocKind::Function);
    entry.signature = Some("(bad`name value)".to_owned());
    entry.group = Some("bad`group".to_owned());
    entry.params = vec![DocParam {
        name: "bad`param".to_owned(),
        summary: "Parameter summary.".to_owned(),
    }];
    entry.requires_capability = vec!["bad`capability".to_owned()];
    entry.see = vec!["bad`see".to_owned()];
    entry.source = Some("bad`source.scm".to_owned());
    entry.range = Some(SourceRange {
        start: SourcePosition {
            line: 2,
            character: 4,
        },
        end: SourcePosition {
            line: 2,
            character: 8,
        },
    });

    let summary_table = entry_summary_markdown_table([&entry]);
    let detailed = detailed_markdown_for_entry(&entry);
    let titled = titled_markdown_for_entry(&entry);
    let source = source_markdown_for_entry(&entry).expect("source markdown");

    assert!(summary_table.contains("`` bad`name ``"));
    assert!(detailed.starts_with("## `` bad`name ``"));
    assert!(detailed.contains("`` bad`param ``"));
    assert!(detailed.contains("`` bad`capability ``"));
    assert!(detailed.contains("`` bad`see ``"));
    assert!(titled.starts_with("## `` bad`name ``"));
    assert!(titled.contains("`` bad`source.scm:3 ``"));
    assert!(source.starts_with("## `` bad`name `` source"));
    assert!(source.contains("`` bad`source.scm:3 ``"));
    assert!(!detailed.contains("`bad`name`"));
}

#[test]
fn source_markdown_returns_none_without_source() {
    let entry = DocEntry::new("demo", DocKind::Function);

    assert_eq!(source_markdown_for_entry(&entry), None);
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

#[test]
fn doc_entry_reports_group_fallback_and_source_location() {
    let mut entry = DocEntry::new("demo", DocKind::Function);

    assert_eq!(entry.group_name(), "Language");
    assert_eq!(entry.display_source_location(), None);

    entry.group = Some("Fixtures".to_owned());
    entry.source = Some("fixture.scm".to_owned());
    entry.range = Some(SourceRange {
        start: SourcePosition {
            line: 2,
            character: 4,
        },
        end: SourcePosition {
            line: 2,
            character: 8,
        },
    });

    assert_eq!(entry.group_name(), "Fixtures");
    assert_eq!(
        entry.display_source_location().as_deref(),
        Some("fixture.scm:3")
    );
}

#[cfg(feature = "reference")]
#[test]
fn search_doc_entries_uses_ranked_fuzzy_matching() {
    let index = DocIndex::scaffold();
    let matches = search_doc_entries(&index, "ctlg tool", 20);

    assert!(matches.iter().any(|entry| entry.name == "catalog/tool"));
    assert_eq!(
        search_doc_entries(&index, "tool", 5)
            .first()
            .map(|entry| entry.name.as_str()),
        Some("tool")
    );
    assert!(
        search_doc_entries(&index, "context read only", 5)
            .iter()
            .any(|entry| entry.name == "source/path")
    );
    assert!(
        search_doc_entries(&index, "ripgrep", 5)
            .iter()
            .any(|entry| entry.name == "catalog/tool")
    );
    assert!(
        search_doc_entries(&index, "tool/path", 5)
            .iter()
            .any(|entry| entry.name == "tool/path")
    );
    assert!(
        search_doc_entries(&index, "src/dsl/std/catalog/tool.scm:16", 5)
            .iter()
            .any(|entry| entry.name == "tool")
    );
    assert!(
        search_doc_entries(&index, "src/dsl/std/catalog/tool.scm:16:1", 5)
            .iter()
            .any(|entry| entry.name == "tool")
    );
    assert!(
        search_doc_entries(&index, "ctlg", 5)
            .iter()
            .any(|entry| entry.name == "catalog")
    );
    assert!(
        !search_doc_entries(&index, "zzzzzzz", 20)
            .iter()
            .any(|entry| entry.name == "tool")
    );
    assert!(search_doc_entries(&index, "nope", 20).is_empty());
}

#[cfg(feature = "reference")]
#[test]
fn suggest_doc_entries_uses_conservative_symbolic_typos() {
    let index = DocIndex::scaffold();

    assert_eq!(
        suggest_doc_entries(&index, "catalgo", 5)
            .first()
            .map(|entry| entry.name.as_str()),
        Some("catalog")
    );
    assert_eq!(
        suggest_doc_entries(&index, "sourcpath", 5)
            .first()
            .map(|entry| entry.name.as_str()),
        Some("source/path")
    );
    assert_eq!(
        suggest_doc_entries(&index, "catlgtool", 5)
            .first()
            .map(|entry| entry.name.as_str()),
        Some("catalog/tool")
    );
    assert!(suggest_doc_entries(&index, "zzzzzzz", 5).is_empty());
    assert!(suggest_doc_entries(&index, "no-such-query", 5).is_empty());
}

#[test]
fn search_doc_entries_matches_lifecycle_metadata() {
    let mut index = DocIndex::empty();
    let mut entry = DocEntry::new("lifecycle-demo", DocKind::Function);
    entry.since = Some("1.2.3".to_owned());
    entry.stability = Some("experimental".to_owned());
    index.insert(entry);

    assert_eq!(
        search_doc_entries(&index, "experimental", 5)
            .first()
            .map(|entry| entry.name.as_str()),
        Some("lifecycle-demo")
    );
    assert_eq!(
        search_doc_entries(&index, "1.2.3", 5)
            .first()
            .map(|entry| entry.name.as_str()),
        Some("lifecycle-demo")
    );
}

#[test]
fn search_prose_fields_deduplicate_keyword_summary_markdown() {
    let index = DocIndex::with_language_keywords();
    let entry = index.get("begin").expect("begin keyword");
    let fields = search::reference_entry_prose_search_fields(entry);

    assert_eq!(
        fields
            .iter()
            .filter(|field| field.as_str()
                == "Evaluate expressions in order and return the final value.")
            .count(),
        1
    );
}
