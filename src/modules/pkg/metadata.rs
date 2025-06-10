use super::super::pkg;
use crate::dprintln;
use crate::modules::project;
use crate::modules::system::path;
use crate::utils::archive::extract_archive; // archive.rsからインポート
use cmd_arg::cmd_arg::{Option, OptionType};
use std::env;
use std::fs;

pub fn metadata(args: Vec<&Option>) -> Result<(), std::io::Error> {
    let mut target_path_str = String::new();
    for arg in args {
        match arg.opt_type {
            OptionType::Simple => target_path_str = arg.opt_str.to_owned(),
            _ => continue,
        }
    }
    let target_path = env::current_dir()?.join(&target_path_str);

    if !target_path.is_file() {
        eprintln!("Couldn't find target file: {}", target_path.display());
        return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
    }

    let cache_path = path::local::cache_path();

    if cache_path.is_file() {
        fs::remove_file(&cache_path)?;
    }
    if cache_path.is_dir() && cache_path.read_dir()?.next().is_some() {
        fs::remove_dir_all(&cache_path)?;
    }
    if !cache_path.is_dir() {
        fs::create_dir_all(&cache_path)?;
    }

    let pkg_archive_in_cache =
        cache_path.join(target_path.file_name().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Target path has no filename",
            )
        })?);
    fs::copy(&target_path, &pkg_archive_in_cache)?;

    dprintln!(
        "Created cache for {} at {}",
        target_path.display(),
        pkg_archive_in_cache.display()
    );

    // アーカイブを展開（tarコマンドの代わりにextract_archiveを使用）
    dprintln!(
        "Trying to extract: from {}, to {}",
        pkg_archive_in_cache.display(),
        cache_path.display()
    );
    extract_archive(pkg_archive_in_cache.clone(), cache_path.clone())?;

    fs::remove_file(&pkg_archive_in_cache)?;

    let pkg_filename_str = pkg_archive_in_cache
        .file_name()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid package archive path (no filename component)",
            )
        })?
        .to_str()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid package file name (non-UTF8)",
            )
        })?;

    let parts: Vec<&str> = pkg_filename_str.split('.').collect();
    let extracted_dir_name = if parts.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Package filename resulted in empty parts",
        ));
    } else if parts.len() == 1 {
        parts[0].to_string()
    } else {
        parts[0..parts.len() - 1].join(".")
    };

    let extracted_pkg_dir_in_cache = cache_path.join(&extracted_dir_name);

    let metadata_process_result = {
        let original_cwd = env::current_dir()?;
        if !extracted_pkg_dir_in_cache.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Extracted package directory not found: {}. Expected after extracting {}.",
                    extracted_pkg_dir_in_cache.display(),
                    pkg_filename_str
                ),
            ));
        }
        env::set_current_dir(&extracted_pkg_dir_in_cache)?;
        dprintln!(
            "Changed current directory to {}",
            extracted_pkg_dir_in_cache.display()
        );

        let result = metadata_process();

        env::set_current_dir(&original_cwd)?;
        dprintln!(
            "Restored current directory to {}",
            original_cwd.display()
        );
        result
    };
    let pkg_data = metadata_process_result?;
    println!("{}", pkg_data);
    Ok(())
}

fn metadata_process() -> Result<pkg::PackageData, std::io::Error> {
    project::metadata::metadata().map_err(std::io::Error::other)?;
    project::metadata::metadata()
}
