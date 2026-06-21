#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallEvent {
    pub tool: String,
    pub action: InstallEventKind,
    pub detail: String,
}

impl InstallEvent {
    pub(crate) fn new(
        tool: impl Into<String>,
        action: InstallEventKind,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            tool: tool.into(),
            action,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallEventKind {
    Present,
    Skip,
    Run,
    Extract,
    Remove,
}

impl InstallEventKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Skip => "skip",
            Self::Run => "run",
            Self::Extract => "extract",
            Self::Remove => "remove",
        }
    }
}

pub(super) fn command_detail(argv: &[String]) -> String {
    shlex::try_join(argv.iter().map(String::as_str)).unwrap_or_else(|_err| argv.join(" "))
}

pub trait InstallReporter {
    fn report(&mut self, event: InstallEvent);
}

#[derive(Debug, Default)]
pub struct NoopReporter;

impl InstallReporter for NoopReporter {
    fn report(&mut self, _event: InstallEvent) {}
}

#[cfg(test)]
mod tests {
    use super::{InstallEventKind, command_detail};

    #[test]
    fn event_kind_labels_are_stable_lowercase_values() {
        assert_eq!(InstallEventKind::Present.label(), "present");
        assert_eq!(InstallEventKind::Skip.label(), "skip");
        assert_eq!(InstallEventKind::Run.label(), "run");
        assert_eq!(InstallEventKind::Extract.label(), "extract");
        assert_eq!(InstallEventKind::Remove.label(), "remove");
    }

    #[test]
    fn command_detail_quotes_shell_arguments() {
        let argv = vec![
            "pkg".to_owned(),
            "install".to_owned(),
            "demo tool".to_owned(),
            "author's tool".to_owned(),
        ];

        assert_eq!(
            command_detail(&argv),
            "pkg install 'demo tool' \"author's tool\""
        );
    }
}
