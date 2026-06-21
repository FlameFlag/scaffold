use serde::Deserialize;

use scaffold_platform::Host;

use super::{ArchiveAction, PackageAction};

#[derive(Debug, Deserialize, strum::IntoStaticStr, strum::VariantNames)]
#[serde(tag = "type", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Action {
    Required,
    Package(PackageAction),
    Build(BuildAction),
    Archive(ArchiveAction),
}

impl Action {
    #[must_use]
    pub fn label(&self) -> &'static str {
        self.into()
    }

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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Deserialize,
    strum::IntoStaticStr,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Phase {
    Prerequisites,
    Packages,
    Builds,
}

impl Phase {
    #[must_use]
    pub fn label(self) -> &'static str {
        self.into()
    }
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
        argvs_with_fallback(&self.argvs, &self.argv)
    }
}

pub(super) fn argvs_with_fallback<'a>(
    argvs: &'a [Vec<String>],
    argv: &'a [String],
) -> Vec<&'a [String]> {
    if !argvs.is_empty() {
        argvs.iter().map(Vec::as_slice).collect()
    } else if !argv.is_empty() {
        vec![argv]
    } else {
        Vec::new()
    }
}
