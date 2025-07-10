use crate::dprintln;
use crate::{modules::pkg::PackageData, utils::files::is_file_exists};
use std::{env, io, path::PathBuf}; 






pub fn get_dir() -> Result<PathBuf, io::Error> {
    let mut current_path = env::current_dir()?; 
    loop {
        let metadata_path = current_path.join("ipak/project.yaml");
        dprintln!("{}", metadata_path.display()); 
        if is_file_exists(metadata_path.to_str().ok_or_else(|| {
            
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid path characters",
            )
        })?) {
            return Ok(current_path);
        } else {
            dprintln!(
                "Not found project.yaml in {}",
                current_path.display()
            );
            if let Some(parent) = current_path.parent() {
                current_path = parent.to_owned(); 
            } else {
                
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "project.yaml not found in current or parent directories",
                ));
            }
        }
    }
}






pub fn get_path() -> Result<PathBuf, io::Error> {
    get_dir().map(|dir| dir.join("ipak/project.yaml"))
}






pub fn metadata() -> Result<PackageData, io::Error> {
    let metadata_path = get_path()?; 
    let read_data =
        std::fs::read_to_string(&metadata_path).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "Failed to read {}: {}",
                    metadata_path.display(),
                    e
                ),
            )
        })?;

    serde_yaml::from_str::<PackageData>(&read_data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse {}: {}", metadata_path.display(), e),
        )
    })
}






pub fn show_metadata() -> Result<(), io::Error> {
    
    let package_data = from_current()?;
    println!("{}", package_data);
    Ok(())
}











pub fn write(package_data: &PackageData) -> Result<(), io::Error> {
    let metadata_path = get_path()?; 

    
    
    let parent_dir = metadata_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not determine parent directory for project.yaml",
        )
    })?;
    std::fs::create_dir_all(parent_dir)?; 

    
    let yaml_string =
        serde_yaml::to_string(package_data).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize PackageData to YAML: {}", e),
            )
        })?;

    
    
    std::fs::write(&metadata_path, yaml_string).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "Failed to write to {}: {}",
                metadata_path.display(),
                e
            ),
        )
    })?;

    dprintln!(
        "Successfully wrote project metadata to {}",
        metadata_path.display()
    );
    Ok(())
}






pub fn from_current() -> Result<PackageData, io::Error> {
    let current_dir = env::current_dir()?;
    let metadata_path = current_dir.join("ipak/project.yaml");

    dprintln!("Attempting to read from: {}", metadata_path.display());

    
    if !is_file_exists(metadata_path.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid path characters in current directory",
        )
    })?) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "'ipak/project.yaml' not found in current directory: {}",
                current_dir.display()
            ),
        ));
    }

    
    let read_data =
        std::fs::read_to_string(&metadata_path).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "Failed to read {}: {}",
                    metadata_path.display(),
                    e
                ),
            )
        })?;

    
    serde_yaml::from_str::<PackageData>(&read_data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse {}: {}", metadata_path.display(), e),
        )
    })
}












pub fn to_current(package_data: &PackageData) -> Result<(), io::Error> {
    let current_dir = env::current_dir()?;
    let metadata_path = current_dir.join("ipak/project.yaml");

    
    let parent_dir = metadata_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not determine parent directory for ipak/project.yaml",
        )
    })?;
    std::fs::create_dir_all(parent_dir)?;

    dprintln!("Attempting to write to: {}", metadata_path.display());

    
    let yaml_string =
        serde_yaml::to_string(package_data).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize PackageData to YAML: {}", e),
            )
        })?;

    
    
    std::fs::write(&metadata_path, yaml_string).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "Failed to write to {}: {}",
                metadata_path.display(),
                e
            ),
        )
    })?;

    dprintln!(
        "Successfully wrote project metadata to {}",
        metadata_path.display()
    );
    Ok(())
}
