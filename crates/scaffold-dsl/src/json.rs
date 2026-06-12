use scheme_rs::{
    lists::{List, Pair},
    symbols::Symbol,
    syntax::Syntax,
    value::{UnpackedValue, Value},
    vectors::Vector,
};
use serde_json::{Map, Number};

use super::{DslError, Result};

pub(super) fn top_level_forms(syntax: &Syntax) -> Result<&[Syntax]> {
    let Some([forms @ .., end]) = syntax.as_list() else {
        return Err(DslError::Shape {
            path: "$".to_owned(),
            message: "expected a Scheme source file".to_owned(),
        });
    };
    if !end.is_null() {
        return Err(DslError::Shape {
            path: "$".to_owned(),
            message: "expected a proper top-level form list".to_owned(),
        });
    }
    Ok(forms)
}

pub(super) fn value_is_null(value: &Value) -> bool {
    matches!(value.clone().unpack(), UnpackedValue::Null)
}

pub(super) fn is_top_level_syntax_definition(form: &Syntax) -> bool {
    form.as_list()
        .and_then(|items| items.first())
        .and_then(Syntax::as_ident)
        .is_some_and(|ident| ident == "define-syntax")
}

pub(super) fn value_to_json(value: Value, path: &str) -> Result<serde_json::Value> {
    match value.clone().unpack() {
        UnpackedValue::Null => Ok(serde_json::Value::Null),
        UnpackedValue::Boolean(value) => Ok(serde_json::Value::Bool(value)),
        UnpackedValue::Character(value) => Ok(serde_json::Value::String(value.to_string())),
        UnpackedValue::Number(_) => number_to_json(value, path),
        UnpackedValue::String(value) => Ok(serde_json::Value::String(value.into())),
        UnpackedValue::Symbol(value) => Ok(serde_json::Value::String(value.to_string())),
        UnpackedValue::Vector(value) => vector_to_json(value, path),
        UnpackedValue::Pair(value) => pair_to_json(value, path),
        other => Err(DslError::Shape {
            path: path.to_owned(),
            message: format!("unsupported Scheme value type {}", other.type_name()),
        }),
    }
}

fn number_to_json(value: Value, path: &str) -> Result<serde_json::Value> {
    if let Ok(value) = i64::try_from(&value) {
        return Ok(serde_json::Value::Number(Number::from(value)));
    }
    if let Ok(value) = u64::try_from(&value) {
        return Ok(serde_json::Value::Number(Number::from(value)));
    }
    if let Ok(value) = f64::try_from(&value)
        && let Some(number) = Number::from_f64(value)
    {
        return Ok(serde_json::Value::Number(number));
    }

    Err(DslError::Shape {
        path: path.to_owned(),
        message: "number cannot be represented as a JSON number".to_owned(),
    })
}

fn vector_to_json(value: Vector, path: &str) -> Result<serde_json::Value> {
    value
        .iter()
        .enumerate()
        .map(|(index, value)| value_to_json(value, &format!("{path}[{index}]")))
        .collect::<Result<Vec<_>>>()
        .map(serde_json::Value::Array)
}

fn pair_to_json(value: Pair, path: &str) -> Result<serde_json::Value> {
    let pair_value = Value::from(value);
    let values = pair_value
        .cast_to_scheme_type::<List>()
        .ok_or_else(|| DslError::Shape {
            path: path.to_owned(),
            message: "expected a proper list or association list".to_owned(),
        })?
        .into_vec();
    if let Some(object) = object_from_alist(&values, path)? {
        return Ok(serde_json::Value::Object(object));
    }

    values
        .into_iter()
        .enumerate()
        .map(|(index, value)| value_to_json(value, &format!("{path}[{index}]")))
        .collect::<Result<Vec<_>>>()
        .map(serde_json::Value::Array)
}

fn object_from_alist(
    values: &[Value],
    path: &str,
) -> Result<Option<Map<String, serde_json::Value>>> {
    let mut object = Map::new();
    for (index, value) in values.iter().enumerate() {
        let UnpackedValue::Pair(pair) = value.clone().unpack() else {
            return Ok(None);
        };
        let (key, entry_value) = pair.into();
        let Some(key) = object_key(key) else {
            return Ok(None);
        };
        let _previous = object.insert(key, value_to_json(entry_value, &format!("{path}.{index}"))?);
    }
    Ok(Some(object))
}

fn object_key(value: Value) -> Option<String> {
    match value.unpack() {
        UnpackedValue::String(value) => Some(value.into()),
        UnpackedValue::Symbol(value) => Some(symbol_key(value)),
        _ => None,
    }
}

fn symbol_key(value: Symbol) -> String {
    value.to_string().replace('-', "_")
}
