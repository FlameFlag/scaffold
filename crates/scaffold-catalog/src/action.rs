use serde::Deserialize;

use scaffold_platform::Host;

use super::{ArchiveAction, PackageAction};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Required,
    Package(PackageAction),
    Build(BuildAction),
    Archive(ArchiveAction),
}

impl Action {
    #[must_use]
    pub const fn phase(&self) -> Phase {
        match self {
            Self::Required => Phase::Prerequisites,
            Self::Package(_) => Phase::Packages,
            Self::Build(_) => Phase::Builds,
            Self::Archive(_) => Phase::Builds,
        }
    }

    #[must_use]
    pub fn supports_host(&self, host: Host) -> bool {
        match self {
            Self::Package(action) => action.supports_host(host),
            Self::Required | Self::Build(_) | Self::Archive(_) => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Prerequisites,
    Packages,
    Builds,
}

#[derive(Debug, Deserialize)]
pub struct BuildAction {
    pub path: String,
    #[serde(default)]
    pub argv: Vec<String>,
    #[serde(default)]
    pub argvs: Vec<Vec<String>>,
}

impl BuildAction {
    pub const fn apply_defaults(&mut self) {}

    pub fn command_argvs(&self) -> Vec<&[String]> {
        if !self.argvs.is_empty() {
            self.argvs.iter().map(Vec::as_slice).collect()
        } else if !self.argv.is_empty() {
            vec![self.argv.as_slice()]
        } else {
            Vec::new()
        }
    }
}
