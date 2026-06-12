#[must_use]
pub fn missing_doc_stub(name: &str, indent: &str) -> String {
    format!("{indent}(doc-next\n{indent}  (summary \"Describe `{name}`.\"))\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_missing_doc_stub_with_definition_indent() {
        assert_eq!(
            missing_doc_stub("local-helper", "  "),
            concat!(
                "  (doc-next\n",
                "    (summary \"Describe `local-helper`.\"))\n\n"
            )
        );
    }
}
