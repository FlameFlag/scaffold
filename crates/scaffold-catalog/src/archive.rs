use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ArchiveAction {
    pub path: String,
    #[serde(default)]
    pub strip_components: usize,
}

impl ArchiveAction {
    pub const fn apply_defaults(&mut self) {}
}
