use scheme_rs::{
    exceptions::Exception,
    strings::WideString,
    syntax::{
        Syntax,
        parse::{LexerError, ParseSyntaxError},
    },
    value::Value,
};

pub fn parse_source(source: &str, source_name: &str) -> Result<Syntax, ParseSyntaxError> {
    Syntax::from_str(source, Some(source_name))
}

pub fn parse_error_offset(error: &ParseSyntaxError, source: &str) -> usize {
    match error {
        ParseSyntaxError::ExpectedClosingParen { span }
        | ParseSyntaxError::UnexpectedClosingParen { span }
        | ParseSyntaxError::InvalidPeriodLocation { span }
        | ParseSyntaxError::NonByte { span }
        | ParseSyntaxError::UnclosedParen { span } => span.offset,
        ParseSyntaxError::Lex(
            LexerError::InvalidCharacterInHexEscape { span, .. }
            | LexerError::UnexpectedCharacter { span, .. }
            | LexerError::BadEscapeCharacter { span, .. },
        ) => span.offset,
        ParseSyntaxError::UnexpectedToken { token } => token.span.offset,
        ParseSyntaxError::UnexpectedEof | ParseSyntaxError::Lex(LexerError::UnexpectedEof) => {
            source.len().saturating_sub(1)
        }
        ParseSyntaxError::CharTryFrom(_)
        | ParseSyntaxError::Lex(LexerError::ReadError(_))
        | ParseSyntaxError::ParseNumberError(_) => 0,
    }
}

#[must_use]
pub fn source_position_byte_offset(source: &str, line: u32, column: usize) -> usize {
    // scheme-rs spans are 1-indexed by line and 0-indexed by column.
    let line_start = source
        .split_inclusive('\n')
        .take(line.saturating_sub(1) as usize)
        .map(str::len)
        .sum::<usize>();
    line_start
        + source[line_start..]
            .char_indices()
            .nth(column)
            .map_or(0, |(offset, _)| offset)
}

pub const fn parse_error_is_incomplete(error: &ParseSyntaxError) -> bool {
    matches!(
        error,
        ParseSyntaxError::UnexpectedEof
            | ParseSyntaxError::ExpectedClosingParen { .. }
            | ParseSyntaxError::UnclosedParen { .. }
            | ParseSyntaxError::Lex(LexerError::UnexpectedEof)
    )
}

pub fn proper_list(syntax: &Syntax) -> Option<&[Syntax]> {
    let items = syntax.as_list()?;
    let (end, body) = items.split_last()?;
    end.is_null().then_some(body)
}

pub fn identifier_text(syntax: &Syntax) -> Option<String> {
    syntax.as_ident().map(|ident| ident.symbol().to_string())
}

pub fn is_identifier(syntax: &Syntax, name: &str) -> bool {
    syntax.as_ident().is_some_and(|ident| ident == name)
}

pub fn wrapped_string_text(syntax: &Syntax) -> Option<String> {
    let Syntax::Wrapped { value, .. } = syntax else {
        return None;
    };
    value.cast_to_scheme_type::<WideString>().map(Into::into)
}

pub fn value_to_string(value: &Value) -> Result<String, Exception> {
    value.try_to_scheme_type::<WideString>().map(Into::into)
}

#[must_use]
pub fn escape_string_literal_body(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[must_use]
pub fn string_literal(value: &str) -> String {
    let escaped = escape_string_literal_body(value);
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incomplete_parse_points_at_end_of_source() {
        let source = "(define x 1";
        let error = parse_source(source, "test.scm").expect_err("source is incomplete");

        assert_eq!(parse_error_offset(&error, source), source.len() - 1);
        assert!(parse_error_is_incomplete(&error));
    }

    #[test]
    fn proper_list_excludes_parser_sentinel() {
        let syntax = parse_source("(define x 1)", "test.scm").expect("source parses");
        let forms = proper_list(&syntax).expect("top-level source is a proper list");

        assert_eq!(forms.len(), 1);
    }

    #[test]
    fn source_position_byte_offset_uses_scheme_span_coordinates() {
        let source = "λ\n(define café 1)";

        assert_eq!(source_position_byte_offset(source, 1, 0), 0);
        assert_eq!(
            source_position_byte_offset(source, 2, 8),
            source.find("café").expect("identifier appears")
        );
    }

    #[test]
    fn string_literal_escapes_scheme_string_delimiters() {
        assert_eq!(string_literal("bin/demo"), "\"bin/demo\"");
        assert_eq!(escape_string_literal_body("bin/demo"), "bin/demo");
        assert_eq!(
            string_literal("C:\\Tools\\\"demo\""),
            "\"C:\\\\Tools\\\\\\\"demo\\\"\""
        );
        assert_eq!(
            escape_string_literal_body("C:\\Tools\\\"demo\""),
            "C:\\\\Tools\\\\\\\"demo\\\""
        );
    }
}
