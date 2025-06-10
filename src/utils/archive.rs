use crate::dprintln;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::{fmt::Display, path::PathBuf};
use tar::Builder as TarBuilder;
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
    match archive_type {
        ArchiveType::Zip => {
            let file = File::create(&to)?;
            let mut zip = ZipWriter::new(file);
            for entry in WalkDir::new(&from) {
                let entry = entry?;
                let path = entry.path();
                let relative = path
                    .strip_prefix(&from)
                    .map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            e,
                        )
                    })?
                    .to_str()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid path encoding",
                        )
                    })?
                    .replace("\\", "/");
                if path.is_file() {
                    let mut f = File::open(path)?;
                    let options: zip::write::FileOptions<()> =
                        zip::write::FileOptions::default()
                            .compression_method(
                                zip::CompressionMethod::Stored,
                            );
                    zip.start_file(&relative, options)?;
                    std::io::copy(&mut f, &mut zip)?;
                } else if path.is_dir() {
                    zip.add_directory::<&str,zip::write::ExtendedFileOptions>(
                        &relative,
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
            add_directory_contents(&mut builder, &from)?;
            builder.finish()?;
            Ok(())
        }
        ArchiveType::TarGz => {
            let file = File::create(&to)?;
            let encoder = GzEncoder::new(file, Compression::default());
            let mut builder = TarBuilder::new(encoder);
            add_directory_contents(&mut builder, &from)?;
            builder.into_inner()?.finish()?;
            Ok(())
        }
        ArchiveType::TarXz => {
            let file = File::create(&to)?;
            let encoder = XzEncoder::new(file, 6);
            let mut builder = TarBuilder::new(encoder);
            add_directory_contents(&mut builder, &from)?;
            builder.into_inner()?.finish()?;
            Ok(())
        }
        ArchiveType::TarZstd => {
            let file = File::create(&to)?;
            let encoder = ZstdEncoder::new(file, 0)?;
            let mut builder = TarBuilder::new(encoder);
            add_directory_contents(&mut builder, &from)?;
            let encoder = builder.into_inner()?;
            let file = encoder.finish()?;
            drop(file);
            Ok(())
        }
    }
}

// ディレクトリの内容をtarアーカイブに追加
fn add_directory_contents<B: Write>(
    builder: &mut TarBuilder<B>,
    from: &Path,
) -> Result<(), std::io::Error> {
    for entry in WalkDir::new(from) {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(from).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
        })?;
        let name = relative
            .to_str()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid path encoding",
                )
            })?
            .replace("\\", "/");
        if path.is_dir() {
            let metadata = path.metadata()?;
            let mut header = tar::Header::new_gnu();
            header.set_path(&name)?;
            header.set_metadata(&metadata);
            header.set_entry_type(tar::EntryType::Directory);
            builder.append(&header, &mut std::io::empty())?;
        } else if path.is_file() {
            let mut f = File::open(path)?;
            builder.append_file(name, &mut f)?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::super::archive::{
        ArchiveType, create_archive, extract_archive,
    };
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempdir::TempDir;

    // テスト用のディレクトリとファイルを作成
    fn setup_test_dir() -> TempDir {
        let temp_dir = TempDir::new("archive_test")
            .expect("Failed to create temp dir");

        // テストディレクトリ内にサブディレクトリとファイルを作成
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).expect("Failed to create subdir");

        // テストファイル1: rootにテキストファイル
        let file1 = temp_dir.path().join("file1.txt");
        let mut f1 = File::create(&file1).expect("Failed to create file1");
        writeln!(f1, "This is file1 content")
            .expect("Failed to write to file1");

        // テストファイル2: サブディレクトリにテキストファイル
        let file2 = sub_dir.join("file2.txt");
        let mut f2 = File::create(&file2).expect("Failed to create file2");
        writeln!(f2, "This is file2 content")
            .expect("Failed to write to file2");

        temp_dir
    }

    // ディレクトリの内容を比較
    fn compare_dirs(original: &PathBuf, extracted: &PathBuf) -> bool {
        for entry in walkdir::WalkDir::new(original) {
            let entry = entry.unwrap();
            let rel_path = entry.path().strip_prefix(original).unwrap();
            let extracted_path = extracted.join(rel_path);

            if entry.path().is_file() {
                if !extracted_path.exists() {
                    return false;
                }
                let orig_content =
                    fs::read_to_string(entry.path()).unwrap();
                let ext_content =
                    fs::read_to_string(&extracted_path).unwrap();
                if orig_content != ext_content {
                    return false;
                }
            } else if entry.path().is_dir() && !extracted_path.exists() {
                return false;
            }
        }
        true
    }

    // 各アーカイブ形式をテスト
    fn test_archive_type(archive_type: ArchiveType, extension: &str) {
        let temp_dir = setup_test_dir();
        let source_dir = temp_dir.path().to_path_buf();
        let archive_path =
            temp_dir.path().join(format!("test.{}", extension));
        let extract_dir = temp_dir.path().join("extracted");

        // アーカイブ作成
        create_archive(
            source_dir.clone(),
            archive_path.clone(),
            archive_type,
        )
        .expect("Failed to create archive");

        // アーカイブファイルが存在することを確認
        assert!(archive_path.exists(), "Archive file was not created");

        // アーカイブ展開
        fs::create_dir(&extract_dir)
            .expect("Failed to create extract dir");
        extract_archive(archive_path, extract_dir.clone())
            .expect("Failed to extract archive");

        // 元のディレクトリと展開されたディレクトリを比較
        assert!(
            compare_dirs(&source_dir, &extract_dir),
            "Extracted directory does not match original for {}",
            extension
        );
    }

    #[test]
    fn test_zip_archive() {
        test_archive_type(ArchiveType::Zip, "zip");
    }

    #[test]
    fn test_tar_archive() {
        test_archive_type(ArchiveType::Tar, "tar");
    }

    #[test]
    fn test_targz_archive() {
        test_archive_type(ArchiveType::TarGz, "tar.gz");
    }

    #[test]
    fn test_tarxz_archive() {
        test_archive_type(ArchiveType::TarXz, "tar.xz");
    }

    #[test]
    fn test_tarzstd_archive() {
        test_archive_type(ArchiveType::TarZstd, "tar.zst");
    }

    #[test]
    fn test_invalid_archive_type() {
        let temp_dir = setup_test_dir();
        let invalid_archive = temp_dir.path().join("test.invalid");
        let extract_dir = temp_dir.path().join("extracted");

        // 無効な拡張子のアーカイブを展開
        let result = extract_archive(invalid_archive, extract_dir);
        assert!(
            result.is_err(),
            "Expected error for invalid archive type"
        );
        if let Err(e) = result {
            assert_eq!(
                e.kind(),
                std::io::ErrorKind::InvalidInput,
                "Expected InvalidInput error"
            );
        }
    }

    #[test]
    fn test_nonexistent_source() {
        let temp_dir = setup_test_dir();
        let nonexistent = temp_dir.path().join("nonexistent");
        let archive_path = temp_dir.path().join("test.zip");

        // 存在しないソースディレクトリでアーカイブ作成
        let result =
            create_archive(nonexistent, archive_path, ArchiveType::Zip);
        assert!(result.is_err(), "Expected error for nonexistent source");
    }
}
