use std::collections::BTreeSet;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;

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

    let paths = env::var_os("PATH")?;
    env::split_paths(&paths)
        .map(|dir| dir.join(executable_name(bin)))
        .find(|path| is_command_file(path))
}

pub fn available_commands() -> Vec<String> {
    let Some(paths) = env::var_os("PATH") else {
        return Vec::new();
    };
    let mut commands = BTreeSet::new();
    for dir in env::split_paths(&paths) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if !is_command_file(&path) {
                continue;
            }
            let Some(name) = command_name(&path) else {
                continue;
            };
            let _inserted = commands.insert(name);
        }
    }
    commands.into_iter().collect()
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
        let root = unique_test_dir("process-run-in");
        std::fs::create_dir_all(&root).expect("root");

        run_in(
            Some(&root),
            &[
                shell.to_string_lossy().into_owned(),
                "-c".to_owned(),
                "pwd > pwd.txt".to_owned(),
            ],
        )
        .expect("run in cwd");

        let pwd = std::fs::read_to_string(root.join("pwd.txt")).expect("pwd");
        assert_eq!(
            canonical_test_path(Path::new(pwd.trim())),
            canonical_test_path(&root)
        );
        drop(std::fs::remove_dir_all(root));
    }

    #[test]
    fn resolves_direct_executable_path() {
        let root = unique_test_dir("process-path-of");
        std::fs::create_dir_all(&root).expect("root");
        let executable = root.join(executable_name("demo"));
        std::fs::write(&executable, "#!/bin/sh\nexit 0\n").expect("executable");
        make_executable(&executable);

        assert_eq!(
            path_of(executable.to_string_lossy().as_ref()),
            Some(executable.clone())
        );
        drop(std::fs::remove_dir_all(root));
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

    fn unique_test_dir(name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "scaffold-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
    }

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
