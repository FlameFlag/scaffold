use serde::Deserialize;

use scaffold_platform::{Host, HostOs, Predicate};
use scaffold_process as process;

use super::action::argvs_with_fallback;

#[derive(Debug, Deserialize)]
pub struct PackageAction {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub install_argv: Vec<String>,
    #[serde(default)]
    pub install_argvs: Vec<Vec<String>>,
    #[serde(default)]
    pub platforms: Vec<PackagePlatform>,
}

impl PackageAction {
    pub fn apply_defaults(&mut self, tool_name: &str) {
        if self.name.is_empty() {
            self.name = tool_name.to_owned();
        }
        for platform in &mut self.platforms {
            if platform.name.is_none() {
                platform.name = Some(self.name.clone());
            }
        }
    }

    pub fn for_host(&self, host: Host) -> Option<ResolvedPackage<'_>> {
        if let Some(platform) = self
            .platforms
            .iter()
            .find(|platform| platform.matches(host))
        {
            return Some(ResolvedPackage {
                name: platform.name.as_deref().unwrap_or(&self.name),
                install_argvs: platform.install_argvs(),
            });
        }

        let install_argvs = argvs_with_fallback(&self.install_argvs, &self.install_argv);
        if install_argvs.is_empty() {
            return None;
        };

        Some(ResolvedPackage {
            name: &self.name,
            install_argvs,
        })
    }

    #[must_use]
    pub fn supports_host(&self, host: Host) -> bool {
        self.for_host(host).is_some()
    }

    #[must_use]
    pub fn inferred_platforms(&self) -> Vec<HostOs> {
        if !self.install_argv.is_empty() || !self.install_argvs.is_empty() {
            return Vec::new();
        }

        let mut platforms = Vec::new();
        for platform in &self.platforms {
            let Some(os) = platform.when.os else {
                return Vec::new();
            };
            if !platforms.contains(&os) {
                platforms.push(os);
            }
        }
        platforms
    }
}

pub struct ResolvedPackage<'a> {
    pub name: &'a str,
    pub install_argvs: Vec<&'a [String]>,
}

#[derive(Debug, Deserialize)]
pub struct PackagePlatform {
    pub when: Predicate,
    #[serde(default)]
    pub requires_commands: Vec<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub install_argv: Vec<String>,
    #[serde(default)]
    pub install_argvs: Vec<Vec<String>>,
}

impl PackagePlatform {
    fn matches(&self, host: Host) -> bool {
        host.matches(self.when)
            && self
                .requires_commands
                .iter()
                .all(|command| process::path_of(command).is_some())
    }

    fn install_argvs(&self) -> Vec<&[String]> {
        argvs_with_fallback(&self.install_argvs, &self.install_argv)
    }
}
