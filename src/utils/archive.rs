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
