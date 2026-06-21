use lexpr::datum::{Datum, Ref};

use scaffold_editor::sexpr;

pub(super) fn library_name(datums: &[Datum]) -> Option<Vec<String>> {
    let [datum] = datums else {
        return None;
    };
    let items = sexpr::list_items(datum.as_ref())?;
    let head = items.first().and_then(|item| sexpr::symbol_text(*item))?;
    (head == "library").then_some(())?;
    items.get(1).and_then(|item| library_name_from_datum(*item))
}

pub(super) fn imported_libraries(datums: &[Datum]) -> Vec<Vec<String>> {
    if let [datum] = datums
        && let Some(items) = sexpr::list_items(datum.as_ref())
        && items
            .first()
            .and_then(|item| sexpr::symbol_text(*item))
            .is_some_and(|head| head == "library")
    {
        imported_libraries_from_declarations(items.into_iter().skip(2))
    } else {
        imported_libraries_from_declarations(datums.iter().map(|datum| datum.as_ref()))
    }
}

fn imported_libraries_from_declarations<'a>(
    declarations: impl IntoIterator<Item = Ref<'a>>,
) -> Vec<Vec<String>> {
    declarations
        .into_iter()
        .filter_map(sexpr::list_items)
        .filter(|items| {
            items
                .first()
                .and_then(|item| sexpr::symbol_text(*item))
                .is_some_and(|head| head == "import")
        })
        .flat_map(|items| {
            items
                .into_iter()
                .skip(1)
                .filter_map(library_name_from_datum)
        })
        .collect()
}

fn library_name_from_datum(datum: Ref<'_>) -> Option<Vec<String>> {
    sexpr::list_items(datum)?
        .into_iter()
        .map(sexpr::symbol_text)
        .map(|component| component.map(str::to_owned))
        .collect()
}
