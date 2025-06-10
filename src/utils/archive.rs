use crate::dprintln;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::{Builder as TarBuilder, Header};
use walkdir::WalkDir;
use xz2::write::XzEncoder;
use zip::ZipWriter;
use zstd::stream::Encoder as ZstdEncoder;

#[derive(Default)]
pub enum ArchiveType {
    Zip,
    #[default]
    TarGz,
    TarXz,
    TarZstd,
    Tar,
}

impl Display for ArchiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zip => write!(f, "zip"),
            Self::TarGz => write!(f, "tar.gz"),
            Self::TarXz => write!(f, "tar.xz"),
            Self::TarZstd => write!(f, "tar.zst"),
            Self::Tar => write!(f, "tar"),
        }
    }
}

// ファイル拡張子からアーカイブタイプを判定
fn get_archive_type(path: &Path) -> Option<ArchiveType> {
    let path_str = path.to_str()?;
    if path_str.ends_with(".zip") {
        Some(ArchiveType::Zip)
    } else if path_str.ends_with(".tar") {
        Some(ArchiveType::Tar)
    } else if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
        Some(ArchiveType::TarGz)
    } else if path_str.ends_with(".tar.xz") {
        Some(ArchiveType::TarXz)
    } else if path_str.ends_with(".tar.zst") {
        Some(ArchiveType::TarZstd)
    } else {
        None
    }
}

pub fn extract_archive(
    from: PathBuf,
    to: PathBuf,
) -> Result<(), std::io::Error> {
    dprintln!(
        "Extracting archive from: {} to: {}",
        from.display(),
        to.display()
    );
    let archive_type = get_archive_type(&from).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Unknown archive type",
        )
    })?;
    let file = File::open(&from)?;
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
            archive.unpack(&to)?;
            Ok(())
        }
    }
}

pub fn create_archive(
    from: PathBuf,
    to: PathBuf,
    archive_type: ArchiveType,
) -> Result<(), std::io::Error> {
    dprintln!(
        "Creating archive from: {} to: {} with type: {}",
        from.display(),
        to.display(),
        archive_type
    );

    // 末尾スラッシュの有無をチェック
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

    // ディレクトリ名の取得（スラッシュなしの場合に使用）
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
            let file = File::create(&to)?;
            let mut zip = ZipWriter::new(file);
            for entry in WalkDir::new(&from) {
                let entry = entry?;
                let path = entry.path();
                let relative = path.strip_prefix(&from).map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        e,
                    )
                })?;
                // パスを構築
                let name = if has_slash {
                    // スラッシュあり: 中身を直接アーカイブ
                    relative
                        .to_str()
                        .ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid path encoding",
                            )
                        })?
                        .replace("\\", "/")
                } else {
                    // スラッシュなし: ディレクトリ名をプレフィックスに
                    if relative == Path::new("") {
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
                                .replace("\\", "/")
                        )
                    }
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
            let file = File::create(&to)?;
            let mut builder = TarBuilder::new(file);
            add_directory_contents(&mut builder, &from, dir_name)?;
            builder.finish()?;
            Ok(())
        }
        ArchiveType::TarGz => {
            let file = File::create(&to)?;
            let encoder = GzEncoder::new(file, Compression::default());
            let mut builder = TarBuilder::new(encoder);
            add_directory_contents(&mut builder, &from, dir_name)?;
            builder.into_inner()?.finish()?;
            Ok(())
        }
        ArchiveType::TarXz => {
            let file = File::create(&to)?;
            let encoder = XzEncoder::new(file, 6);
            let mut builder = TarBuilder::new(encoder);
            add_directory_contents(&mut builder, &from, dir_name)?;
            builder.into_inner()?.finish()?;
            Ok(())
        }
        ArchiveType::TarZstd => {
            let file = File::create(&to)?;
            let encoder = ZstdEncoder::new(file, 0)?;
            let mut builder = TarBuilder::new(encoder);
            add_directory_contents(&mut builder, &from, dir_name)?;
            let encoder = builder.into_inner()?;
            let file = encoder.finish()?;
            drop(file);
            Ok(())
        }
    }
}

// ディレクトリの内容をtarアーカイブに追加（ディレクトリ名をオプションで使用）
fn add_directory_contents<B: Write>(
    builder: &mut TarBuilder<B>,
    from: &Path,
    dir_name: Option<&str>,
) -> Result<(), std::io::Error> {
    for entry in WalkDir::new(from) {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(from).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Failed to strip prefix: {}", e),
            )
        })?;
        // パスを構築
        let name = if let Some(dir_name) = dir_name {
            // ディレクトリ名をプレフィックスに
            if relative == Path::new("") {
                dir_name.to_string()
            } else {
                format!(
                    "{}/{}",
                    dir_name,
                    relative
                        .to_str()
                        .ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid path encoding",
                            )
                        })?
                        .replace("\\", "/")
                )
            }
        } else {
            // スラッシュあり: 中身を直接アーカイブ
            relative
                .to_str()
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid path encoding",
                    )
                })?
                .replace("\\", "/")
        };
        if path.is_dir() {
            let metadata = path.metadata()?;
            let mut header = Header::new_ustar();
            // 長いパスをサポートするため、名前を直接設定
            if name.len() <= 100 {
                header.set_path(&name)?;
            } else {
                // 長いパスは名前フィールドに収まらない場合、簡略化またはエラー
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Path too long for tar: {}", name),
                ));
            }
            header.set_entry_type(tar::EntryType::Directory);
            header.set_size(0);
            if cfg!(unix) {
                header.set_metadata(&metadata);
            } else {
                header.set_mtime(
                    metadata
                        .modified()?
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(|e| std::io::Error::other(e))?
                        .as_secs(),
                );
                header.set_mode(0o755);
            }
            builder.append(&header, &mut std::io::empty())?;
        } else if path.is_file() {
            let metadata = path.metadata()?;
            let mut f = File::open(path)?;
            let mut header = Header::new_ustar();
            if name.len() <= 100 {
                header.set_path(&name)?;
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Path too long for tar: {}", name),
                ));
            }
            header.set_entry_type(tar::EntryType::Regular);
            header.set_size(metadata.len());
            #[cfg(unix)]
            if cfg!(unix) {
                header.set_metadata(&metadata);
            } else {
                header.set_mtime(
                    metadata
                        .modified()?
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(|e| std::io::Error::other(e))?
                        .as_secs(),
                );
                header.set_mode(0o644);
            }
            builder.append(&header, &mut f)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn test_tar_gz_with_slash() {
        let temp_dir = TempDir::new("archive_test").unwrap();
        let source_dir = temp_dir.path().join("dir-a");
        fs::create_dir(&source_dir).unwrap();
        let file1 = source_dir.join("text.txt");
        File::create(&file1).unwrap().write_all(b"Test content").unwrap();

        let archive_path = temp_dir.path().join("test.tar.gz");
        let source_dir_with_slash =
            PathBuf::from(format!("{}/", source_dir.to_str().unwrap()));
        create_archive(
            source_dir_with_slash,
            archive_path.clone(),
            ArchiveType::TarGz,
        )
        .unwrap();

        let extract_dir = temp_dir.path().join("extracted");
        fs::create_dir(&extract_dir).unwrap();
        extract_archive(archive_path, extract_dir.clone()).unwrap();

        assert!(
            extract_dir.join("text.txt").exists(),
            "text.txt not found in root"
        );
        assert!(
            !extract_dir.join("dir-a").is_dir(),
            "dir-a should not be extracted"
        );
    }

    #[test]
    fn test_tar_gz_long_path_error() {
        let temp_dir = TempDir::new("archive_test").unwrap();
        let source_dir = temp_dir.path().join("dir-a");
        fs::create_dir(&source_dir).unwrap();
        let long_file = source_dir.join(
            "a".repeat(150), // 100バイトを超えるパス
        );
        File::create(&long_file)
            .unwrap()
            .write_all(b"Test content")
            .unwrap();

        let archive_path = temp_dir.path().join("test.tar.gz");
        let result = create_archive(
            source_dir.clone(),
            archive_path.clone(),
            ArchiveType::TarGz,
        );
        assert!(result.is_err(), "Expected error for long path");
    }
}
