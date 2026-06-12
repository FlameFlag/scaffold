use serde::{Deserialize, de};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostOs {
    Linux,
    Macos,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostArch {
    Aarch64,
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

fn parse_predicate(value: &str) -> Result<Predicate, String> {
    let (os, arch) = value
        .split_once('-')
        .map_or((value, None), |(os, arch)| (os, Some(arch)));
    let os = match os {
        "macos" | "darwin" => HostOs::Macos,
        "linux" => HostOs::Linux,
        "windows" | "win32" => HostOs::Windows,
        _ => return Err(format!("unknown host OS predicate {os:?}")),
    };
    let arch = match arch {
        None => None,
        Some("aarch64" | "arm64") => Some(HostArch::Aarch64),
        Some("x86_64" | "amd64" | "x64") => Some(HostArch::X86_64),
        Some(arch) => return Err(format!("unknown host architecture predicate {arch:?}")),
    };
    Ok(Predicate { os: Some(os), arch })
}
