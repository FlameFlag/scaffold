use serde::Deserialize;
use serde_json::{Map, Value};

use scaffold_platform::{Host, HostOs, Predicate};
use scaffold_process as process;

use super::{Action, CatalogError, Phase};

#[derive(Debug, Deserialize)]
pub struct Tool {
    pub name: String,
    #[serde(default)]
    pub bins: Vec<Bin>,
    #[serde(default)]
    pub paths: Vec<ToolPath>,
    #[serde(default)]
    pub checks: Vec<ToolCheck>,
    #[serde(default)]
    pub platforms: Vec<HostOs>,
    #[serde(default)]
    pub requires: Vec<Predicate>,
    #[serde(default)]
    pub depends: Vec<String>,
    #[serde(default)]
    pub before: Vec<String>,
    #[serde(default)]
    pub after: Vec<String>,
    #[serde(default)]
    pub order: Option<i32>,
    #[serde(default)]
    pub meta: ToolMeta,
    #[serde(default)]
    pub passthru: Map<String, Value>,
    #[serde(default)]
    pub uninstall: UninstallPlan,
    #[serde(default = "default_verify_after_install")]
    pub verify_after_install: bool,
    #[serde(default)]
    pub phase: Option<Phase>,
    pub action: Action,
}

#[derive(Debug, Default, Deserialize)]
pub struct ToolMeta {
    #[serde(default)]
    pub home_page: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub maintainers: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub main_program: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

impl Tool {
    pub fn apply_defaults(&mut self) -> Result<(), CatalogError> {
        if self.name.is_empty() {
            return Err(CatalogError::Invalid(
                "tool name cannot be empty".to_owned(),
            ));
        }
        if self.bins.is_empty() {
            self.bins.push(Bin {
                name: self.name.clone(),
                version_argv: Vec::new(),
            });
        }
        match &mut self.action {
            Action::Required => {}
            Action::Package(action) => action.apply_defaults(&self.name),
            Action::Build(action) => action.apply_defaults(),
            Action::Archive(action) => action.apply_defaults(),
        }
        if self.platforms.is_empty()
            && let Action::Package(action) = &self.action
        {
            self.platforms = action.inferred_platforms();
        }
        Ok(())
    }

    #[must_use]
    pub fn supports_host(&self, host: Host) -> bool {
        (self.platforms.is_empty() || self.platforms.contains(&host.os))
            && self
                .requires
                .iter()
                .copied()
                .all(|predicate| host.matches(predicate))
            && self.action.supports_host(host)
    }

    #[must_use]
    pub fn phase(&self) -> Phase {
        self.phase.unwrap_or_else(|| self.action.phase())
    }

    pub fn required_paths_for_host(&self, host: Host) -> impl Iterator<Item = &str> {
        self.paths
            .iter()
            .filter(move |path| host.matches(path.when))
            .map(|path| path.path.as_str())
    }

    pub fn checks_for_host(&self, host: Host) -> impl Iterator<Item = &[String]> {
        self.checks
            .iter()
            .filter(move |check| host.matches(check.when))
            .map(|check| check.argv.as_slice())
    }

    pub fn version_summary(&self) -> String {
        self.bins
            .iter()
            .filter_map(Bin::version)
            .collect::<Vec<_>>()
            .join(", ")
    }

    #[must_use]
    pub fn wants_first(&self) -> bool {
        self.before
            .iter()
            .any(|target| matches!(target.as_str(), "none" | "first"))
    }
}

const fn default_verify_after_install() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct ToolPath {
    #[serde(default)]
    pub when: Predicate,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct ToolCheck {
    #[serde(default)]
    pub when: Predicate,
    pub argv: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct UninstallPlan {
    #[serde(default)]
    pub commands: Vec<UninstallCommand>,
    #[serde(default)]
    pub paths: Vec<ToolPath>,
    #[serde(default)]
    pub remove_bins: Option<bool>,
    #[serde(default)]
    pub remove_prefix: Option<bool>,
}

impl UninstallPlan {
    pub fn commands_for_host(&self, host: Host) -> impl Iterator<Item = &[String]> {
        self.commands
            .iter()
            .filter(move |command| host.matches(command.when))
            .map(|command| command.argv.as_slice())
    }

    pub fn paths_for_host(&self, host: Host) -> impl Iterator<Item = &str> {
        self.paths
            .iter()
            .filter(move |path| host.matches(path.when))
            .map(|path| path.path.as_str())
    }
}

#[derive(Debug, Deserialize)]
pub struct UninstallCommand {
    #[serde(default)]
    pub when: Predicate,
    pub argv: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Bin {
    pub name: String,
    #[serde(default)]
    pub version_argv: Vec<String>,
}

impl Bin {
    fn version(&self) -> Option<String> {
        let argv = if self.version_argv.is_empty() {
            vec![self.name.clone(), "--version".to_owned()]
        } else {
            self.version_argv.clone()
        };
        let output = process::capture(&argv).ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8(output.stdout)
            .or_else(|_| String::from_utf8(output.stderr))
            .ok()?;
        let first_line = text.lines().next()?.trim();
        (!first_line.is_empty()).then(|| first_line.to_owned())
    }
}
