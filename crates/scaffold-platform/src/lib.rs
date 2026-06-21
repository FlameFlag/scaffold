use serde::{Deserialize, Serialize, de};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Host {
    pub os: HostOs,
    pub arch: HostArch,
}

impl Host {
    #[must_use]
    pub const fn current() -> Self {
        Self {
            os: HostOs::current(),
            arch: HostArch::current(),
        }
    }

    #[must_use]
    pub fn matches(self, predicate: Predicate) -> bool {
        predicate.os.is_none_or(|os| os == self.os)
            && predicate.arch.is_none_or(|arch| arch == self.arch)
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    Deserialize,
    Serialize,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum HostOs {
    Linux,
    #[strum(to_string = "macos", serialize = "darwin")]
    Macos,
    #[strum(to_string = "windows", serialize = "win32")]
    Windows,
}

impl HostOs {
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::Linux
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        self.into()
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    Deserialize,
    Serialize,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum HostArch {
    #[strum(to_string = "aarch64", serialize = "arm64")]
    Aarch64,
    #[strum(to_string = "x86_64", serialize = "amd64", serialize = "x64")]
    X86_64,
}

impl HostArch {
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(target_arch = "aarch64") {
            Self::Aarch64
        } else {
            Self::X86_64
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        self.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Predicate {
    pub os: Option<HostOs>,
    pub arch: Option<HostArch>,
}

impl<'de> Deserialize<'de> for Predicate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum PredicateRepr {
            Short(String),
            Full {
                os: Option<HostOs>,
                arch: Option<HostArch>,
            },
        }

        match PredicateRepr::deserialize(deserializer)? {
            PredicateRepr::Short(value) => parse_predicate(&value).map_err(de::Error::custom),
            PredicateRepr::Full { os, arch } => Ok(Self { os, arch }),
        }
    }
}

pub fn parse_predicate(value: &str) -> Result<Predicate, String> {
    let (os, arch) = value
        .split_once('-')
        .map_or((value, None), |(os, arch)| (os, Some(arch)));
    let os = os
        .parse()
        .map_err(|_err| format!("unknown host OS predicate {os:?}"))?;
    let arch = arch
        .map(|arch| {
            arch.parse()
                .map_err(|_err| format!("unknown host architecture predicate {arch:?}"))
        })
        .transpose()?;
    Ok(Predicate { os: Some(os), arch })
}

#[cfg(test)]
mod tests {
    use super::{HostArch, HostOs, parse_predicate};
    use strum::VariantNames as _;

    #[test]
    fn host_labels_are_stable_catalog_predicate_strings() {
        assert_eq!(HostOs::Linux.label(), "linux");
        assert_eq!(HostOs::Macos.label(), "macos");
        assert_eq!(HostOs::Windows.label(), "windows");
        assert_eq!(HostArch::Aarch64.label(), "aarch64");
        assert_eq!(HostArch::X86_64.label(), "x86_64");
        assert_eq!(HostOs::VARIANTS, ["linux", "macos", "windows"]);
        assert_eq!(HostArch::VARIANTS, ["aarch64", "x86_64"]);
    }

    #[test]
    fn host_serializes_to_stable_platform_labels() {
        let value = serde_json::to_value(super::Host {
            os: HostOs::Macos,
            arch: HostArch::Aarch64,
        })
        .expect("host serializes");

        assert_eq!(value["os"], "macos");
        assert_eq!(value["arch"], "aarch64");
    }

    #[test]
    fn short_predicates_accept_catalog_aliases() {
        assert_eq!(
            parse_predicate("darwin-arm64").expect("darwin arm64"),
            super::Predicate {
                os: Some(HostOs::Macos),
                arch: Some(HostArch::Aarch64)
            }
        );
        assert_eq!(
            parse_predicate("win32-amd64").expect("win32 amd64"),
            super::Predicate {
                os: Some(HostOs::Windows),
                arch: Some(HostArch::X86_64)
            }
        );
        assert_eq!(
            parse_predicate("linux-x64").expect("linux x64"),
            super::Predicate {
                os: Some(HostOs::Linux),
                arch: Some(HostArch::X86_64)
            }
        );
    }

    #[test]
    fn short_predicate_errors_keep_existing_wording() {
        assert_eq!(
            parse_predicate("freebsd").expect_err("invalid os"),
            "unknown host OS predicate \"freebsd\""
        );
        assert_eq!(
            parse_predicate("linux-riscv64").expect_err("invalid arch"),
            "unknown host architecture predicate \"riscv64\""
        );
    }
}
