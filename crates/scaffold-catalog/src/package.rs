use serde::Deserialize;

use scaffold_platform::{Host, Predicate};
use scaffold_process as process;

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

        let install_argvs = if !self.install_argvs.is_empty() {
            self.install_argvs.iter().map(Vec::as_slice).collect()
        } else if !self.install_argv.is_empty() {
            vec![self.install_argv.as_slice()]
        } else {
            return None;
        };

        Some(ResolvedPackage {
            name: &self.name,
            install_argvs,
        })
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
        if !self.install_argvs.is_empty() {
            self.install_argvs.iter().map(Vec::as_slice).collect()
        } else if !self.install_argv.is_empty() {
            vec![self.install_argv.as_slice()]
        } else {
            Vec::new()
        }
    }
}
