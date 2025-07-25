//! このモジュールは、様々な形式のアーカイブ（zip, tar.gz, tar.xz, tar.zstd, tar, unix ar）の作成と展開機能を提供します。
//! ファイルパスの処理とアーカイブタイプに応じた適切な圧縮・解凍ロジックを管理します。

use ar::Archive as ArArchive;
use ar::Builder as ArBuilder;
use clap;
use file_format::{self, FileFormat};
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tar::{Builder as TarBuilder, Header};
use walkdir::WalkDir;
use xz2::write::XzEncoder;
use zip::ZipWriter;
use zstd::stream::Encoder as ZstdEncoder;

#[derive(Default, clap::ValueEnum, Clone, Copy, Debug)]
pub enum ArchiveType {
    Zip,
    #[default]
    TarGz,
    TarXz,
    TarZstd,
    Tar,
    UnixAr,
}

impl Display for ArchiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zip => write!(f, "zip"),
            Self::TarGz => write!(f, "tar.gz"),
            Self::TarXz => write!(f, "tar.xz"),
            Self::TarZstd => write!(f, "tar.zst"),
            Self::Tar => write!(f, "tar"),
            Self::UnixAr => write!(f, "unix archive"),
        }
    }
}

impl FromStr for ArchiveType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "zip" => Ok(ArchiveType::Zip),
            "tar.gz" | "tgz" => Ok(ArchiveType::TarGz),
            "tar.xz" | "txz" => Ok(ArchiveType::TarXz),
            "tar.zst" | "tar.zstd" | "tzst" => Ok(ArchiveType::TarZstd),
            "tar" => Ok(ArchiveType::Tar),
            "ar" => Ok(ArchiveType::UnixAr),
            _ => Err(format!("Invalid Archive Type: {}", s)),
        }
    }
}

pub fn get_archive_type(path: &Path) -> Result<ArchiveType, String> {
    let archive_format = match FileFormat::from_file(path) {
        Ok(file_format) => file_format,
        Err(e) => {
            return Err(format!("Error while getting file format: {}", e));
        }
    };
    let archive_extension = archive_format.extension();
    match archive_extension {
        "zip" => Ok(ArchiveType::Zip),
        "tar" => Ok(ArchiveType::Tar),
        "gz" | "gzip" | "tar.gz" => Ok(ArchiveType::TarGz),
        "xz" | "tar.xz" => Ok(ArchiveType::TarXz),
        "zst" | "zstd" | "tar.zst" | "tar.zstd" => {
            Ok(ArchiveType::TarZstd)
        }
        "deb" | "rpm" | "ar" | "a" => Ok(ArchiveType::UnixAr),
        _ => Err(archive_extension.to_string()),
    }
}

pub fn extract_archive(
    from: &PathBuf,
    to: &PathBuf,
) -> Result<(), std::io::Error> {
    log::debug!(
        "Extracting archive from: {} to: {}",
        from.display(),
        to.display()
    );
    let archive_type = get_archive_type(from).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unknown archive type: {}", e),
        )
    })?;
    let file = File::open(from)?;
    match archive_type {
        ArchiveType::Zip => {
            let mut archive = zip::ZipArchive::new(file)?;
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let outpath = to.join(file.mangled_name());
                if file.name().ends_with('/') {
                    std::fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p)?;
                        }
                    }
                    let mut outfile = File::create(&outpath)?;
                    std::io::copy(&mut file, &mut outfile)?;
                }
            }
            Ok(())
        }
        ArchiveType::UnixAr => {
            let mut archive = ArArchive::new(file);
            while let Some(entry) = archive.next_entry() {
                let mut entry = entry?;
                let header = entry.header();
                let entry_name_bytes = header.identifier();
                let entry_name =
                    String::from_utf8_lossy(entry_name_bytes).into_owned();
                let outpath = to.join(entry_name);

                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut entry, &mut outfile)?;
            }
            Ok(())
        }
        _ => {
            let reader = match archive_type {
                ArchiveType::Tar => Box::new(file) as Box<dyn Read>,
                ArchiveType::TarGz => {
                    Box::new(flate2::read::GzDecoder::new(file))
                        as Box<dyn Read>
                }
                ArchiveType::TarXz => {
                    Box::new(xz2::read::XzDecoder::new(file))
                        as Box<dyn Read>
                }
                ArchiveType::TarZstd => {
                    Box::new(zstd::stream::Decoder::new(file)?)
                        as Box<dyn Read>
                }
                _ => unreachable!(),
            };
            let mut archive = tar::Archive::new(reader);
            archive.unpack(to)?;
            Ok(())
        }
    }
}

pub fn create_archive(
    from: &PathBuf,
    to: &PathBuf,
    archive_type: ArchiveType,
) -> Result<(), std::io::Error> {
    log::debug!(
        "Creating archive from: {} to: {} with type: {}",
        from.display(),
        to.display(),
        archive_type
    );

    let has_slash = from
        .to_str()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Non-UTF8 path",
            )
        })?
        .ends_with('/')
        || from.to_str().unwrap().ends_with('\\');

    let dir_name = if !has_slash {
        Some(
            from.file_name()
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid directory name",
                    )
                })?
                .to_str()
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Non-UTF8 directory name",
                    )
                })?,
        )
    } else {
        None
    };

    match archive_type {
        ArchiveType::Zip => {
            let file = File::create(to)?;
            let mut zip = ZipWriter::new(file);
            for entry in WalkDir::new(from) {
                let entry = entry?;
                let path = entry.path();
                let relative = path.strip_prefix(from).map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        e.to_string(),
                    )
                })?;

                let name = if has_slash {
                    relative
                        .to_str()
                        .ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid path encoding",
                            )
                        })?
                        .replace('\\', "/")
                } else if relative == Path::new("") {
                    dir_name.unwrap().to_string()
                } else {
                    format!(
                        "{}/{}",
                        dir_name.unwrap(),
                        relative
                            .to_str()
                            .ok_or_else(|| {
                                std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    "Invalid path encoding",
                                )
                            })?
                            .replace('\\', "/")
                    )
                };
                if path.is_file() {
                    let mut f = File::open(path)?;
                    let options: zip::write::FileOptions<()> =
                        zip::write::FileOptions::default()
                            .compression_method(
                                zip::CompressionMethod::Stored,
                            );
                    zip.start_file(&name, options)?;
                    std::io::copy(&mut f, &mut zip)?;
                } else if path.is_dir() {
                    if has_slash && relative == Path::new("") {
                        continue;
                    }

                    zip.add_directory::<&str, zip::write::ExtendedFileOptions>(
                        &format!("{}/", name), 
                        zip::write::FileOptions::default(),
                    )?;
                }
            }
            zip.finish()?;
            Ok(())
        }
        ArchiveType::Tar => {
            let file = File::create(to)?;
            let mut builder = TarBuilder::new(file);
            add_directory_contents(
                &mut builder,
                from,
                has_slash,
                dir_name,
            )?;
            builder.finish()?;
            Ok(())
        }
        ArchiveType::TarGz => {
            let file = File::create(to)?;
            let encoder = GzEncoder::new(file, Compression::default());
            let mut builder = TarBuilder::new(encoder);
            add_directory_contents(
                &mut builder,
                from,
                has_slash,
                dir_name,
            )?;
            builder.into_inner()?.finish()?;
            Ok(())
        }
        ArchiveType::TarXz => {
            let file = File::create(to)?;
            let encoder = XzEncoder::new(file, 6);
            let mut builder = TarBuilder::new(encoder);
            add_directory_contents(
                &mut builder,
                from,
                has_slash,
                dir_name,
            )?;
            builder.into_inner()?.finish()?;
            Ok(())
        }
        ArchiveType::TarZstd => {
            let file = File::create(to)?;
            let encoder = ZstdEncoder::new(file, 0)?;
            let mut builder = TarBuilder::new(encoder);
            add_directory_contents(
                &mut builder,
                from,
                has_slash,
                dir_name,
            )?;
            let encoder = builder.into_inner()?;
            let file = encoder.finish()?;
            drop(file);
            Ok(())
        }
        ArchiveType::UnixAr => {
            let file = File::create(to)?;
            let mut builder = ArBuilder::new(file);

            for entry in WalkDir::new(from) {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    let relative_path =
                        path.strip_prefix(from).map_err(|e| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!(
                                    "Failed to strip prefix for ar: {}",
                                    e
                                ),
                            )
                        })?;

                    let ar_name = if has_slash {
                        relative_path
                            .to_str()
                            .ok_or_else(|| {
                                std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    "Non-UTF8 path for ar archive",
                                )
                            })?
                            .to_string()
                    } else {
                        format!(
                            "{}/{}",
                            dir_name.unwrap(),
                            relative_path.to_str().ok_or_else(|| {
                                std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    "Non-UTF8 path for ar archive",
                                )
                            })?
                        )
                    };

                    let mut file_to_archive = File::open(path)?;
                    let metadata = path.metadata()?;

                    let mut header = ar::Header::new(
                        ar_name.into_bytes(),
                        metadata.len(),
                    );
                    header.set_mtime(
                        metadata
                            .modified()?
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    );
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::MetadataExt;
                        header.set_mode(metadata.mode());
                        header.set_uid(metadata.uid());
                        header.set_gid(metadata.gid());
                    }

                    builder.append(&header, &mut file_to_archive)?;
                }
            }
            builder.into_inner()?.flush()?;
            Ok(())
        }
    }
}

fn add_directory_contents<B: Write>(
    builder: &mut TarBuilder<B>,
    from: &Path,
    has_slash: bool,
    dir_name: Option<&str>,
) -> Result<(), std::io::Error> {
    for entry in WalkDir::new(from) {
        let entry = entry?;
        let path = entry.path();
        let metadata = path.metadata()?;

        let relative = path.strip_prefix(from).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Failed to strip prefix: {}", e),
            )
        })?;

        let name = if has_slash {
            if relative == Path::new("") {
                "".to_string()
            } else {
                relative
                    .to_str()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid path encoding",
                        )
                    })?
                    .replace('\\', "/")
            }
        } else if relative == Path::new("") {
            dir_name.unwrap().to_string()
        } else {
            format!(
                "{}/{}",
                dir_name.unwrap(),
                relative
                    .to_str()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid path encoding",
                        )
                    })?
                    .replace('\\', "/")
            )
        };

        if name.is_empty() {
            continue;
        }

        let entry_path_for_append = PathBuf::from(name.clone());

        if path.is_file() {
            builder.append_path_with_name(path, &entry_path_for_append)?;
        } else if path.is_dir() {
            let mut dir_entry_name = name;
            if !dir_entry_name.ends_with('/') {
                dir_entry_name.push('/');
            }

            let mut header = Header::new_ustar();
            header.set_path(&dir_entry_name)?;
            header.set_entry_type(tar::EntryType::Directory);
            header.set_size(0);

            #[cfg(unix)]
            {
                header.set_metadata(&metadata);
            }

            builder.append(&header, &mut std::io::empty())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_tar_gz_with_slash() {
        let temp_dir =
            TempDir::with_prefix("archive_test_tar_gz_slash").unwrap();
        let source_dir = temp_dir.path().join("dir-a");
        fs::create_dir(&source_dir).unwrap();
        let file1 = source_dir.join("text.txt");
        File::create(&file1).unwrap().write_all(b"Test content").unwrap();

        let archive_path = temp_dir.path().join("test.tar.gz");
        let source_dir_with_slash =
            PathBuf::from(format!("{}/", source_dir.to_str().unwrap()));
        create_archive(
            &source_dir_with_slash,
            &archive_path,
            ArchiveType::TarGz,
        )
        .unwrap();

        let extract_dir = temp_dir.path().join("extracted");
        fs::create_dir(&extract_dir).unwrap();
        extract_archive(&archive_path, &extract_dir).unwrap();

        assert!(
            extract_dir.join("text.txt").exists(),
            "text.txt not found in root"
        );
        assert!(
            !extract_dir.join("dir-a").is_dir(),
            "dir-a should not be extracted as a top-level directory"
        );
    }

    #[test]
    fn test_zip_with_slash() {
        let temp_dir =
            TempDir::with_prefix("zip_test_with_slash").unwrap();
        let source_dir = temp_dir.path().join("zip_dir_a");
        fs::create_dir(&source_dir).unwrap();
        let file1 = source_dir.join("zip_text.txt");
        File::create(&file1).unwrap().write_all(b"Zip content").unwrap();

        let archive_path = temp_dir.path().join("test.zip");
        let source_dir_with_slash =
            PathBuf::from(format!("{}/", source_dir.to_str().unwrap()));
        create_archive(
            &source_dir_with_slash,
            &archive_path,
            ArchiveType::Zip,
        )
        .unwrap();

        let extract_dir = temp_dir.path().join("extracted_zip");
        fs::create_dir(&extract_dir).unwrap();
        extract_archive(&archive_path, &extract_dir).unwrap();

        assert!(
            extract_dir.join("zip_text.txt").exists(),
            "zip_text.txt not found in root of extracted zip"
        );
        assert!(
            !extract_dir.join("zip_dir_a").is_dir(),
            "zip_dir_a should not be extracted as a top-level directory from zip"
        );
    }

    #[test]
    fn test_zip_no_slash() {
        let temp_dir = TempDir::with_prefix("zip_test_no_slash").unwrap();
        let source_dir = temp_dir.path().join("zip_dir_b");
        fs::create_dir(&source_dir).unwrap();
        let file1 = source_dir.join("zip_inner_text.txt");
        File::create(&file1)
            .unwrap()
            .write_all(b"Another zip content")
            .unwrap();

        let archive_path = temp_dir.path().join("test_no_slash.zip");
        create_archive(&source_dir, &archive_path, ArchiveType::Zip)
            .unwrap();

        let extract_dir = temp_dir.path().join("extracted_zip_no_slash");
        fs::create_dir(&extract_dir).unwrap();
        extract_archive(&archive_path, &extract_dir).unwrap();

        assert!(
            extract_dir.join("zip_dir_b").is_dir(),
            "zip_dir_b should be extracted as a top-level directory from zip"
        );
        assert!(
            extract_dir
                .join("zip_dir_b")
                .join("zip_inner_text.txt")
                .exists(),
            "zip_inner_text.txt not found inside zip_dir_b"
        );
    }

    #[test]
    fn test_ar_create_and_extract_with_slash() {
        let temp_dir =
            TempDir::with_prefix("archive_test_ar_slash").unwrap();
        let source_dir = temp_dir.path().join("ar-dir-a");
        fs::create_dir(&source_dir).unwrap();
        let file1 = source_dir.join("ar_text.txt");
        File::create(&file1).unwrap().write_all(b"AR content").unwrap();

        let archive_path = temp_dir.path().join("test.ar");
        let source_dir_with_slash =
            PathBuf::from(format!("{}/", source_dir.to_str().unwrap()));
        create_archive(
            &source_dir_with_slash,
            &archive_path,
            ArchiveType::UnixAr,
        )
        .unwrap();

        let extract_dir = temp_dir.path().join("extracted_ar");
        fs::create_dir(&extract_dir).unwrap();
        extract_archive(&archive_path, &extract_dir).unwrap();

        assert!(
            extract_dir.join("ar_text.txt").exists(),
            "ar_text.txt not found in root of extracted ar"
        );
        assert!(
            !extract_dir.join("ar-dir-a").is_dir(),
            "ar-dir-a should not be extracted as a top-level directory from ar"
        );
    }

    #[test]
    fn test_ar_create_and_extract_no_slash() {
        let temp_dir =
            TempDir::with_prefix("archive_test_ar_no_slash").unwrap();
        let source_dir = temp_dir.path().join("ar-dir-b");
        fs::create_dir(&source_dir).unwrap();
        let file1 = source_dir.join("ar_inner_text.txt");
        File::create(&file1)
            .unwrap()
            .write_all(b"Another AR content")
            .unwrap();

        let archive_path = temp_dir.path().join("test_no_slash.ar");
        create_archive(&source_dir, &archive_path, ArchiveType::UnixAr)
            .unwrap();

        let extract_dir = temp_dir.path().join("extracted_ar_no_slash");
        fs::create_dir(&extract_dir).unwrap();
        extract_archive(&archive_path, &extract_dir).unwrap();

        assert!(
            extract_dir.join("ar-dir-b").is_dir(),
            "ar-dir-b should be extracted as a top-level directory from ar"
        );
        assert!(
            extract_dir
                .join("ar-dir-b")
                .join("ar_inner_text.txt")
                .exists(),
            "ar_inner_text.txt not found inside ar-dir-b"
        );
    }
}
