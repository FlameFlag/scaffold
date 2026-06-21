use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

static ANY_COMMAND: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(".+").expect("valid command matcher"));

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("empty command")]
    EmptyCommand,
    #[error("failed to spawn {program}: {source}")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("command failed: {program} ({status})")]
    CommandFailed {
        program: String,
        status: std::process::ExitStatus,
    },
}

pub struct Output {
    pub status: std::process::ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[must_use]
pub fn path_of(bin: &str) -> Option<PathBuf> {
    if bin.contains(std::path::MAIN_SEPARATOR) {
        let path = PathBuf::from(bin);
        return is_command_file(&path).then_some(path);
    }

    which::which(bin).ok()
}

pub fn available_commands() -> Vec<String> {
    which::which_re(&*ANY_COMMAND)
        .map(|paths| {
            paths
                .filter_map(|path| command_name(&path))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect()
        })
        .unwrap_or_default()
}

pub fn run(argv: &[String]) -> Result<(), ProcessError> {
    run_in(None, argv)
}

pub fn capture(argv: &[String]) -> Result<Output, ProcessError> {
    let (program, args) = argv.split_first().ok_or(ProcessError::EmptyCommand)?;
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|source| ProcessError::Spawn {
            program: program.clone(),
            source,
        })?;
    Ok(Output {
        status: output.status,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

pub fn run_in(cwd: Option<&Path>, argv: &[String]) -> Result<(), ProcessError> {
    let (program, args) = argv.split_first().ok_or(ProcessError::EmptyCommand)?;
    let mut command = Command::new(program);
    let command = command.args(args);
    if let Some(cwd) = cwd {
        let _command = command.current_dir(cwd);
    }
    let status = command.status().map_err(|source| ProcessError::Spawn {
        program: program.clone(),
        source,
    })?;

    if status.success() {
        Ok(())
    } else {
        Err(ProcessError::CommandFailed {
            program: program.clone(),
            status,
        })
    }
}

#[must_use]
pub fn executable_name(bin: &str) -> OsString {
    if cfg!(windows) && !bin.ends_with(".exe") {
        format!("{bin}.exe").into()
    } else {
        bin.into()
    }
}

fn command_name(path: &Path) -> Option<String> {
    #[cfg(windows)]
    {
        let extension = path.extension()?.to_string_lossy();
        let executable_extensions =
            env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.BAT;.CMD".to_owned());
        if !executable_extensions
            .split(';')
            .map(|ext| ext.trim_start_matches('.'))
            .any(|ext| extension.eq_ignore_ascii_case(ext))
        {
            return None;
        }
        path.file_stem()
            .map(|name| name.to_string_lossy().into_owned())
    }
    #[cfg(not(windows))]
    {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
    }
}

fn is_command_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        command_name(path).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn rejects_empty_command() {
        assert!(matches!(run(&[]), Err(ProcessError::EmptyCommand)));
        assert!(matches!(capture(&[]), Err(ProcessError::EmptyCommand)));
    }

    #[test]
    fn captures_successful_process_output() {
        let exe = env::current_exe().expect("current exe");
        let output =
            capture(&[exe.to_string_lossy().into_owned(), "--list".to_owned()]).expect("capture");

        assert!(output.status.success());
        assert!(!output.stdout.is_empty());
    }

    #[test]
    fn runs_in_requested_directory() {
        let Some(shell) = path_of("sh") else {
            return;
        };
        let root = tempfile::tempdir().expect("root");

        run_in(
            Some(root.path()),
            &[
                shell.to_string_lossy().into_owned(),
                "-c".to_owned(),
                "pwd > pwd.txt".to_owned(),
            ],
        )
        .expect("run in cwd");

        let pwd = std::fs::read_to_string(root.path().join("pwd.txt")).expect("pwd");
        assert_eq!(
            canonical_test_path(Path::new(pwd.trim())),
            canonical_test_path(root.path())
        );
    }

    #[test]
    fn resolves_direct_executable_path() {
        let root = tempfile::tempdir().expect("root");
        let executable = root.path().join(executable_name("demo"));
        std::fs::write(&executable, "#!/bin/sh\nexit 0\n").expect("executable");
        make_executable(&executable);

        assert_eq!(
            path_of(executable.to_string_lossy().as_ref()),
            Some(executable.clone())
        );
    }

    #[cfg(unix)]
    fn make_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = std::fs::metadata(path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).expect("permissions");
    }

    #[cfg(not(unix))]
    fn make_executable(_path: &Path) {}

    fn canonical_test_path(path: &Path) -> PathBuf {
        let canonical = std::fs::canonicalize(path).expect("canonical path");
        #[cfg(target_os = "macos")]
        {
            if let Ok(stripped) = canonical.strip_prefix("/private") {
                return Path::new("/").join(stripped);
            }
        }
        canonical
    }
}
