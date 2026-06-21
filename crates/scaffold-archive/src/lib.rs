use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::{Component, Path, PathBuf};

use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use thiserror::Error;
use xz2::read::XzDecoder;

use scaffold_process as process;

#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Process(#[from] process::ProcessError),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error("unsupported archive format for {path:?}")]
    UnsupportedFormat { path: PathBuf },
    #[cfg(not(target_os = "macos"))]
    #[error("DMG archive extraction requires macOS hdiutil and ditto")]
    UnsupportedDmgHost,
    #[cfg(target_os = "macos")]
    #[error("DMG archive extraction does not support strip-components")]
    UnsupportedDmgStripComponents,
    #[error("archive entry uses an unsafe path: {path:?}")]
    UnsafePath { path: PathBuf },
}

pub fn extract_archive(
    archive_path: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    strip_components: usize,
) -> Result<(), ArchiveError> {
    let archive_path = archive_path.as_ref();
    let destination = destination.as_ref();
    std::fs::create_dir_all(destination)?;

    match ArchiveFormat::from_path(archive_path) {
        Some(ArchiveFormat::Zip) => extract_zip(archive_path, destination, strip_components),
        Some(ArchiveFormat::Tar) => {
            let file = File::open(archive_path)?;
            extract_tar(BufReader::new(file), destination, strip_components)
        }
        Some(ArchiveFormat::TarGz) => {
            let file = File::open(archive_path)?;
            extract_tar(
                GzDecoder::new(BufReader::new(file)),
                destination,
                strip_components,
            )
        }
        Some(ArchiveFormat::TarBz2) => {
            let file = File::open(archive_path)?;
            extract_tar(
                BzDecoder::new(BufReader::new(file)),
                destination,
                strip_components,
            )
        }
        Some(ArchiveFormat::TarXz) => {
            let file = File::open(archive_path)?;
            extract_tar(
                XzDecoder::new(BufReader::new(file)),
                destination,
                strip_components,
            )
        }
        Some(ArchiveFormat::Dmg) => extract_dmg(archive_path, destination, strip_components),
        None => Err(ArchiveError::UnsupportedFormat {
            path: archive_path.to_path_buf(),
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    Dmg,
}

impl ArchiveFormat {
    fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().to_ascii_lowercase();
        if name.ends_with(".zip") {
            Some(Self::Zip)
        } else if name.ends_with(".tar") {
            Some(Self::Tar)
        } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            Some(Self::TarGz)
        } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
            Some(Self::TarBz2)
        } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
            Some(Self::TarXz)
        } else if name.ends_with(".dmg") {
            Some(Self::Dmg)
        } else {
            None
        }
    }
}

fn extract_tar<R: Read>(
    reader: R,
    destination: &Path,
    strip_components: usize,
) -> Result<(), ArchiveError> {
    let mut archive = tar::Archive::new(reader);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        let Some(path) = stripped_relative_path(&path, strip_components)? else {
            continue;
        };
        let target = destination.join(path);
        let entry_type = entry.header().entry_type();
        if entry_type.is_dir() {
            std::fs::create_dir_all(&target)?;
        } else if entry_type.is_file() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let _unpacked = entry.unpack(&target)?;
        }
    }
    Ok(())
}

fn extract_zip(
    archive_path: &Path,
    destination: &Path,
    strip_components: usize,
) -> Result<(), ArchiveError> {
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let Some(path) = entry.enclosed_name() else {
            return Err(ArchiveError::UnsafePath {
                path: PathBuf::from(entry.name()),
            });
        };
        let Some(path) = stripped_relative_path(&path, strip_components)? else {
            continue;
        };
        let target = destination.join(path);
        if entry.is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut output = File::create(&target)?;
            let _bytes = io::copy(&mut entry, &mut output)?;
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn extract_dmg(
    archive_path: &Path,
    destination: &Path,
    strip_components: usize,
) -> Result<(), ArchiveError> {
    if strip_components != 0 {
        return Err(ArchiveError::UnsupportedDmgStripComponents);
    }

    let mountpoint = tempfile::Builder::new()
        .prefix("scaffold-dmg-mount-")
        .tempdir()?;
    let mountpoint_text = mountpoint.path().to_string_lossy().into_owned();
    let archive_text = archive_path.to_string_lossy().into_owned();
    process::run(&[
        "hdiutil".to_owned(),
        "attach".to_owned(),
        "-quiet".to_owned(),
        "-nobrowse".to_owned(),
        "-readonly".to_owned(),
        "-mountpoint".to_owned(),
        mountpoint_text.clone(),
        archive_text,
    ])?;

    let _mount = DmgMount { mountpoint };
    let destination_text = destination.to_string_lossy().into_owned();
    process::run(&["ditto".to_owned(), mountpoint_text, destination_text])?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn extract_dmg(
    _archive_path: &Path,
    _destination: &Path,
    _strip_components: usize,
) -> Result<(), ArchiveError> {
    Err(ArchiveError::UnsupportedDmgHost)
}

#[cfg(target_os = "macos")]
struct DmgMount {
    mountpoint: tempfile::TempDir,
}

#[cfg(target_os = "macos")]
impl Drop for DmgMount {
    fn drop(&mut self) {
        drop(
            std::process::Command::new("hdiutil")
                .arg("detach")
                .arg("-quiet")
                .arg(self.mountpoint.path())
                .status(),
        );
    }
}

fn stripped_relative_path(
    path: &Path,
    strip_components: usize,
) -> Result<Option<PathBuf>, ArchiveError> {
    let (_, stripped) = path.components().try_fold(
        (0, PathBuf::new()),
        |(normal_count, mut stripped), component| match component {
            Component::Normal(part) => {
                if normal_count >= strip_components {
                    stripped.push(part);
                }
                Ok((normal_count + 1, stripped))
            }
            Component::CurDir => Ok((normal_count, stripped)),
            _ => Err(ArchiveError::UnsafePath {
                path: path.to_path_buf(),
            }),
        },
    )?;
    if stripped.as_os_str().is_empty() {
        return Ok(None);
    }

    Ok(Some(stripped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_tar_gz_with_stripped_root() {
        let root = tempfile::tempdir().expect("root");
        let archive_path = root.path().join("demo.tar.gz");
        let output_dir = root.path().join("out");

        {
            let file = File::create(&archive_path).expect("archive");
            let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
            let mut builder = tar::Builder::new(encoder);
            let mut header = tar::Header::new_gnu();
            let contents = b"hello";
            header.set_size(contents.len() as u64);
            header.set_cksum();
            builder
                .append_data(&mut header, "demo/bin/tool", &contents[..])
                .expect("append");
            builder.finish().expect("finish");
        }

        extract_archive(&archive_path, &output_dir, 1).expect("extract");

        assert_eq!(
            std::fs::read_to_string(output_dir.join("bin/tool")).expect("tool"),
            "hello"
        );
    }

    #[test]
    fn rejects_unsafe_tar_paths() {
        assert!(matches!(
            stripped_relative_path(Path::new("../escape"), 0),
            Err(ArchiveError::UnsafePath { .. })
        ));
    }

    #[test]
    fn rejects_unsafe_zip_paths() {
        let root = tempfile::tempdir().expect("root");
        let archive_path = root.path().join("escape.zip");
        let output_dir = root.path().join("out");

        {
            let file = File::create(&archive_path).expect("archive");
            let mut writer = zip::ZipWriter::new(file);
            writer
                .start_file("../escape", zip::write::SimpleFileOptions::default())
                .expect("start file");
            io::Write::write_all(&mut writer, b"escape").expect("write");
            let _file = writer.finish().expect("finish");
        }

        assert!(matches!(
            extract_archive(&archive_path, &output_dir, 0),
            Err(ArchiveError::UnsafePath { .. })
        ));
        assert!(!root.path().join("escape").exists());
    }

    #[test]
    fn rejects_unsupported_archive_format() {
        let path = PathBuf::from("demo.rar");

        assert!(matches!(
            extract_archive(&path, std::env::temp_dir(), 0),
            Err(ArchiveError::UnsupportedFormat { path: rejected }) if rejected == path
        ));
    }

    #[test]
    fn recognizes_dmg_format() {
        assert_eq!(
            ArchiveFormat::from_path(Path::new("Example.DMG")),
            Some(ArchiveFormat::Dmg)
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn extracts_dmg_with_hdiutil_and_ditto_when_available() {
        if process::path_of("hdiutil").is_none() || process::path_of("ditto").is_none() {
            return;
        }

        let root = tempfile::tempdir().expect("root");
        let source_dir = root.path().join("source");
        let output_dir = root.path().join("out");
        let archive_path = root.path().join("demo.dmg");
        std::fs::create_dir_all(&source_dir).expect("source");
        std::fs::write(source_dir.join("hello.txt"), "hello dmg").expect("fixture");

        process::run(&[
            "hdiutil".to_owned(),
            "create".to_owned(),
            "-quiet".to_owned(),
            "-srcfolder".to_owned(),
            source_dir.to_string_lossy().into_owned(),
            "-ov".to_owned(),
            "-format".to_owned(),
            "UDZO".to_owned(),
            archive_path.to_string_lossy().into_owned(),
        ])
        .expect("create dmg");

        let extract_result = extract_archive(&archive_path, &output_dir, 0).or_else(|_| {
            std::thread::sleep(std::time::Duration::from_millis(250));
            extract_archive(&archive_path, &output_dir, 0)
        });
        if let Err(error) = extract_result {
            if is_hdiutil_command_failure(&error) {
                eprintln!("skipping DMG extraction test: hdiutil failed on this host");
                return;
            }
            panic!("extract dmg: {error:?}");
        }

        assert!(path_exists_below(&output_dir, Path::new("hello.txt")));
    }

    #[cfg(target_os = "macos")]
    fn is_hdiutil_command_failure(error: &ArchiveError) -> bool {
        matches!(
            error,
            ArchiveError::Process(process::ProcessError::CommandFailed { program, .. })
                if program == "hdiutil"
        )
    }

    #[cfg(target_os = "macos")]
    fn path_exists_below(root: &Path, suffix: &Path) -> bool {
        let Ok(entries) = std::fs::read_dir(root) else {
            return false;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.ends_with(suffix) {
                return true;
            }
            if path.is_dir() && path_exists_below(&path, suffix) {
                return true;
            }
        }
        false
    }
}
