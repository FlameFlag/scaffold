use std::fmt;

use pretty::RcDoc;

use super::form::Form;

pub(super) fn render_form(form: &Form, indent: usize, width: usize, out: &mut String) {
    match form {
        Form::Atom(text) | Form::String(text) | Form::Comment(text) => out.push_str(text),
        Form::Quote(prefix, form) => {
            out.push(*prefix);
            render_form(form, indent, width, out);
        }
        Form::List { open, close, items } => {
            if is_library_form(items) {
                render_library_list(*open, *close, items, indent, width, out);
                return;
            }
            if items.is_empty() {
                out.push(*open);
                out.push(*close);
                return;
            }

            let flat = flat_list(*open, *close, items);
            if indent + flat.len() <= width && !items.iter().any(Form::is_block_comment) {
                out.push_str(&flat);
                return;
            }

            if render_special_list(*open, *close, items, indent, width, out) {
                return;
            }

            out.push(*open);
            render_form(&items[0], indent + 1, width, out);
            for item in &items[1..] {
                push_indented_line(out, indent + 2);
                render_form(item, indent + 2, width, out);
            }
            out.push(*close);
        }
    }
}

fn render_special_list(
    open: char,
    close: char,
    items: &[Form],
    indent: usize,
    width: usize,
    out: &mut String,
) -> bool {
    let Some(head) = list_head_from_items(items) else {
        return false;
    };
    let head_items = match head {
        "define" | "define-syntax" | "doc" | "typedoc" | "extern-doc" | "if" | "lambda" | "let"
        | "let*" | "letrec" => 2,
        "moduledoc" | "doc-next" => 1,
        _ => return false,
    };
    render_head_aligned_list(open, close, items, head_items, indent, width, out);
    true
}

fn render_head_aligned_list(
    open: char,
    close: char,
    items: &[Form],
    head_items: usize,
    indent: usize,
    width: usize,
    out: &mut String,
) {
    let head_items = head_items.min(items.len());
    out.push(open);
    let mut line_len = indent + 1;
    for (index, item) in items[..head_items].iter().enumerate() {
        let flat = flat_form(item);
        if index > 0 && line_len + 1 + flat.len() > width {
            push_indented_line(out, indent + 2);
            render_form(item, indent + 2, width, out);
            line_len = indent + 2 + flat.len();
        } else {
            if index > 0 {
                out.push(' ');
                line_len += 1;
            }
            out.push_str(&flat);
            line_len += flat.len();
        }
    }
    for item in &items[head_items..] {
        push_indented_line(out, indent + 2);
        render_form(item, indent + 2, width, out);
    }
    out.push(close);
}

fn render_library_list(
    open: char,
    close: char,
    items: &[Form],
    indent: usize,
    width: usize,
    out: &mut String,
) {
    let ordered = ordered_library_items(items);
    out.push(open);
    render_form(ordered[0], indent + 1, width, out);
    for (index, item) in ordered[1..].iter().enumerate() {
        out.push('\n');
        if index > 0 && needs_library_blank_line(ordered[index], item) {
            out.push('\n');
        }
        out.extend(std::iter::repeat_n(' ', indent + 2));
        render_form(item, indent + 2, width, out);
    }
    out.push(close);
}

fn push_indented_line(out: &mut String, indent: usize) {
    out.push('\n');
    out.extend(std::iter::repeat_n(' ', indent));
}

fn ordered_library_items(items: &[Form]) -> Vec<&Form> {
    let mut ordered = Vec::with_capacity(items.len());
    let mut consumed_docs = vec![false; items.len()];
    ordered.push(&items[0]);

    for item in items.iter().skip(1) {
        if doc_subject(item).is_some() {
            continue;
        }
        if let Some(name) = definition_name(item) {
            for (doc_index, candidate) in items.iter().enumerate().skip(1) {
                if !consumed_docs[doc_index] && doc_subject(candidate) == Some(name) {
                    ordered.push(candidate);
                    consumed_docs[doc_index] = true;
                }
            }
        }
        ordered.push(item);
    }

    for (index, item) in items.iter().enumerate().skip(1) {
        if !consumed_docs[index] && doc_subject(item).is_some() {
            ordered.push(item);
        }
    }

    ordered
}

fn is_library_form(items: &[Form]) -> bool {
    matches!(items.first(), Some(Form::Atom(head)) if head == "library")
}

fn needs_library_blank_line(previous: &Form, current: &Form) -> bool {
    is_library_body_block(previous) || is_library_body_block(current)
}

fn is_library_body_block(form: &Form) -> bool {
    matches!(
        list_head(form),
        Some(
            "define"
                | "define-syntax"
                | "doc"
                | "typedoc"
                | "extern-doc"
                | "doc-next"
                | "moduledoc",
        )
    )
}

fn doc_subject(form: &Form) -> Option<&str> {
    let Form::List { items, .. } = form else {
        return None;
    };
    let head = list_head(form)?;
    if head != "doc" && head != "typedoc" && head != "extern-doc" {
        return None;
    }
    subject_text(items.get(1)?)
}

fn subject_text(form: &Form) -> Option<&str> {
    match form {
        Form::Atom(text) | Form::String(text) => Some(unquote_string(text)),
        Form::Quote('\'', form) => subject_text(form),
        Form::List { items, .. } => match items.as_slice() {
            [Form::Atom(head), subject] if head == "quote" => subject_text(subject),
            _ => None,
        },
        _ => None,
    }
}

fn definition_name(form: &Form) -> Option<&str> {
    let Form::List { items, .. } = form else {
        return None;
    };
    match items.as_slice() {
        [Form::Atom(head), Form::Atom(name), ..] if head == "define" || head == "define-syntax" => {
            Some(name)
        }
        [
            Form::Atom(head),
            Form::List {
                items: signature, ..
            },
            ..,
        ] if head == "define" => match signature.first()? {
            Form::Atom(name) => Some(name),
            _ => None,
        },
        _ => None,
    }
}

fn list_head(form: &Form) -> Option<&str> {
    let Form::List { items, .. } = form else {
        return None;
    };
    list_head_from_items(items)
}

fn list_head_from_items(items: &[Form]) -> Option<&str> {
    match items.first()? {
        Form::Atom(text) => Some(text),
        _ => None,
    }
}

fn unquote_string(text: &str) -> &str {
    text.strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .unwrap_or(text)
}

fn flat_list(open: char, close: char, items: &[Form]) -> String {
    let body = RcDoc::intersperse(items.iter().map(flat_doc), RcDoc::space());
    let doc = RcDoc::text(open.to_string())
        .append(body)
        .append(RcDoc::text(close.to_string()));
    render_flat_doc(doc)
}

fn flat_form(form: &Form) -> String {
    render_flat_doc(flat_doc(form))
}

fn render_flat_doc(doc: RcDoc<'_, ()>) -> String {
    let mut bytes = Vec::new();
    doc.render(usize::MAX, &mut bytes).expect("render to vec");
    String::from_utf8(bytes).expect("formatter emits utf8")
}

fn flat_doc(form: &Form) -> RcDoc<'_, ()> {
    match form {
        Form::Atom(text) | Form::String(text) | Form::Comment(text) => RcDoc::text(text.clone()),
        Form::Quote(prefix, form) => RcDoc::text(prefix.to_string()).append(flat_doc(form)),
        Form::List { open, close, items } => {
            let body = RcDoc::intersperse(items.iter().map(flat_doc), RcDoc::space());
            RcDoc::text(open.to_string())
                .append(body)
                .append(RcDoc::text(close.to_string()))
        }
    }
}

impl fmt::Display for Form {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        render_form(self, 0, crate::DEFAULT_WIDTH, &mut output);
        f.write_str(&output)
    }
}
