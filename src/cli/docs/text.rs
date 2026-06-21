use scaffold_docs::markdown_code_span;

pub(super) fn markdown_try_command(command: impl AsRef<str>) -> String {
    format!("- {}\n", markdown_code_span(command))
}

pub(super) fn shell_arg(value: &str) -> String {
    shlex::try_quote(value)
        .map(std::borrow::Cow::into_owned)
        .unwrap_or_else(|_err| value.to_owned())
}
