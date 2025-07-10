use crate::utils::archive::{create_archive, extract_archive};
use crate::utils::args::{ArchiveCommands, UtilsCommands};
use crate::utils::error::Error;
pub fn utils(args: UtilsCommands) -> Result<(), Error> {
    // 引数がない場合は早期リターン
    match args {
        UtilsCommands::Archive(args) => archive(args)?,
    }
    Ok(())
}
fn archive(args: ArchiveCommands) -> Result<(), Error> {
    match args {
        ArchiveCommands::Create { from, to, archive_type } => {
            create_archive(&from, &to, archive_type)
                .map_err(|e| -> Error { e.into() })?
        }
        ArchiveCommands::Extract { from, to } => {
            extract_archive(&from, &to)
                .map_err(|e| -> Error { e.into() })?
        }
    }
    Ok(())
}
