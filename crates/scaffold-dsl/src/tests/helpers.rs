use std::{collections::BTreeSet, path::PathBuf};

pub(super) fn single_value(text: &str) -> serde_json::Value {
    let values = crate::values_from_str(text).expect("scheme values");
    assert_eq!(values.len(), 1);
    values.into_iter().next().expect("single value")
}

pub(super) fn fixture_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("fixtures")
        .join(path)
}

pub(super) fn extension_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("extensions")
        .join(path)
}

pub(super) fn assert_platform_extension_files(platform: &str, expected: &[&str]) {
    let dir = extension_path(&format!("platform/{platform}"));
    let actual = std::fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("read {}: {err}", dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|err| panic!("read entry in {}: {err}", dir.display()))
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| name.ends_with(".scm"))
        .collect::<BTreeSet<_>>();
    let expected = expected
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();

    assert_eq!(actual, expected, "unexpected {platform} platform helpers");
}

pub(super) fn unique_test_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "scaffold-dsl-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ))
}
