use scheme_rs::{
    lists::{List, Pair},
    symbols::Symbol,
    syntax::Syntax,
    value::{Value, ValueType},
    vectors::Vector,
};
use serde_json::{Map, Number};

use super::{DslError, Result};
use scaffold_scheme::{proper_list, value_to_string};

pub(super) fn top_level_forms(syntax: &Syntax) -> Result<&[Syntax]> {
    proper_list(syntax).ok_or_else(|| DslError::Shape {
        path: "$".to_owned(),
        message: if syntax.as_list().is_some() {
            "expected a proper top-level form list".to_owned()
        } else {
            "expected a Scheme source file".to_owned()
        },
    })
}

pub(super) fn value_is_null(value: &Value) -> bool {
    value.is_null()
}

pub(super) fn is_top_level_syntax_definition(form: &Syntax) -> bool {
    form.as_list()
        .and_then(|items| items.first())
        .and_then(Syntax::as_ident)
        .is_some_and(|ident| ident == "define-syntax")
}

pub(super) fn value_to_json(value: Value, path: &str) -> Result<serde_json::Value> {
    match value.type_of() {
        ValueType::Null => Ok(serde_json::Value::Null),
        ValueType::Boolean => Ok(serde_json::Value::Bool(bool::from(value))),
        ValueType::Character => Ok(serde_json::Value::String(
            value
                .try_to_scheme_type::<char>()
                .map_err(|err| scheme_value_error(path, err))?
                .to_string(),
        )),
        ValueType::Number => number_to_json(value, path),
        ValueType::String => Ok(serde_json::Value::String(
            value_to_string(&value).map_err(|err| scheme_value_error(path, err))?,
        )),
        ValueType::Symbol => Ok(serde_json::Value::String(
            value
                .try_to_scheme_type::<Symbol>()
                .map_err(|err| scheme_value_error(path, err))?
                .to_string(),
        )),
        ValueType::Vector => vector_to_json(
            value
                .try_to_scheme_type::<Vector>()
                .map_err(|err| scheme_value_error(path, err))?,
            path,
        ),
        ValueType::Pair => pair_to_json(
            value
                .try_to_scheme_type::<Pair>()
                .map_err(|err| scheme_value_error(path, err))?,
            path,
        ),
        _ => Err(DslError::Shape {
            path: path.to_owned(),
            message: format!("unsupported Scheme value type {}", value.type_name()),
        }),
    }
}

fn scheme_value_error(path: &str, error: scheme_rs::exceptions::Exception) -> DslError {
    DslError::Shape {
        path: path.to_owned(),
        message: error.to_string(),
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
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let Some((key, entry_value)) = value.cast_to_scheme_type::<(Value, Value)>() else {
                return Ok(None);
            };
            let Some(key) = object_key(key) else {
                return Ok(None);
            };
            Ok(Some((
                key,
                value_to_json(entry_value, &format!("{path}.{index}"))?,
            )))
        })
        .collect()
}

fn object_key(value: Value) -> Option<String> {
    match value.type_of() {
        ValueType::String => value_to_string(&value).ok(),
        ValueType::Symbol => value.cast_to_scheme_type::<Symbol>().map(symbol_key),
        _ => None,
    }
}

fn symbol_key(value: Symbol) -> String {
    value.to_string().replace('-', "_")
}
