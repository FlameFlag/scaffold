use super::{FormatError, Result, form::Form};

pub(super) struct Parser<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub(super) const fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }

    pub(super) fn parse_all(mut self) -> Result<Vec<Form>> {
        let mut forms = Vec::new();
        while self.skip_ws() {
            forms.push(self.parse_form()?);
        }
        Ok(forms)
    }

    fn parse_form(&mut self) -> Result<Form> {
        let Some(ch) = self.peek() else {
            return Ok(Form::Atom(String::new()));
        };
        match ch {
            '(' => self.parse_list('(', ')'),
            '[' => self.parse_list('[', ']'),
            ')' | ']' => Err(FormatError::UnexpectedClose {
                found: ch,
                offset: self.pos,
            }),
            '"' => self.parse_string(),
            ';' => Ok(self.parse_comment()),
            '\'' | '`' => self.parse_quote(ch),
            ',' => {
                if self.remaining().starts_with(",@") {
                    self.pos += 2;
                    self.parse_quoted(',', "@")
                } else {
                    self.parse_quote(',')
                }
            }
            _ => Ok(self.parse_atom()),
        }
    }

    fn parse_list(&mut self, open: char, close: char) -> Result<Form> {
        self.pos += open.len_utf8();
        let mut items = Vec::new();

        loop {
            let _skipped = self.skip_ws();
            let Some(ch) = self.peek() else {
                return Err(FormatError::UnclosedList { expected: close });
            };
            if ch == close {
                self.pos += close.len_utf8();
                return Ok(Form::List { open, close, items });
            }
            if (ch == ')' || ch == ']') && ch != close {
                return Err(FormatError::UnexpectedClose {
                    found: ch,
                    offset: self.pos,
                });
            }
            items.push(self.parse_form()?);
        }
    }

    fn parse_string(&mut self) -> Result<Form> {
        let start = self.pos;
        self.pos += 1;
        let mut escaped = false;
        while let Some(ch) = self.peek() {
            self.pos += ch.len_utf8();
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                return Ok(Form::String(self.text[start..self.pos].to_owned()));
            }
        }
        Err(FormatError::UnterminatedString { offset: start })
    }

    fn parse_comment(&mut self) -> Form {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.pos += ch.len_utf8();
        }
        Form::Comment(self.text[start..self.pos].trim_end().to_owned())
    }

    fn parse_quote(&mut self, prefix: char) -> Result<Form> {
        self.pos += prefix.len_utf8();
        self.parse_quoted(prefix, "")
    }

    fn parse_quoted(&mut self, prefix: char, suffix: &str) -> Result<Form> {
        let offset = self.pos;
        let _skipped = self.skip_ws();
        if self.peek().is_none() {
            return Err(FormatError::MissingQuotedExpression { offset });
        }
        let form = self.parse_form()?;
        if suffix.is_empty() {
            Ok(Form::Quote(prefix, Box::new(form)))
        } else {
            Ok(Form::Quote(
                prefix,
                Box::new(Form::Quote('@', Box::new(form))),
            ))
        }
    }

    fn parse_atom(&mut self) -> Form {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || matches!(ch, '(' | ')' | '[' | ']' | '"' | ';') {
                break;
            }
            self.pos += ch.len_utf8();
        }
        Form::Atom(self.text[start..self.pos].to_owned())
    }

    fn skip_ws(&mut self) -> bool {
        while let Some(ch) = self.peek() {
            if !ch.is_whitespace() {
                return true;
            }
            self.pos += ch.len_utf8();
        }
        false
    }

    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn remaining(&self) -> &'a str {
        &self.text[self.pos..]
    }
}
