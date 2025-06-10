use std::{fmt::Display, path::PathBuf};

use crate::dprintln;

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

pub fn extract_archive(
    from: PathBuf,
    to: PathBuf,
) -> Result<(), std::io::Error> {
    dprintln!(
        "Extracting archive from: {} to: {}",
        from.display(),
        to.display()
    );
    Ok(())
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
    Ok(())
}
