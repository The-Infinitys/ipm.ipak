use super::ExecMode;
use super::ExecShell;
use super::metadata; 
use crate::dprintln;
use crate::utils::color::colorize::*;
use std::fmt::{self, Display};

#[derive(Default)]
pub struct RemoveOptions {
    
    pub remove_shell: ExecShell,
    pub remove_mode: ExecMode,
}

impl Display for RemoveOptions {
    
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", "Remove Options".cyan().bold())?;
        writeln!(
            f,
            "  {}: {}",
            "remove-shell".green().bold(),
            self.remove_shell
        )?;
        writeln!(
            f,
            "  {}: {}",
            "remove-mode".green().bold(),
            self.remove_mode
        )?;
        Ok(())
    }
}














pub fn remove(opts: RemoveOptions) -> Result<(), String> {
    dprintln!("{}", &opts); 

    
    
    let target_dir = metadata::get_dir().map_err(|_| {
        "Error: Couldn't find Ipak Directory. Make sure you are in a project directory or Ipak is installed."
            .to_string()
    })?;

    
    
    let project_metadata = metadata::metadata() 
        .map_err(|e| {
            format!("Error: Failed to retrieve project metadata: {:?}", e)
        })?;

    
    let mut remove_process = opts.remove_shell.generate();

    
    remove_process
        .current_dir(&target_dir)
        .env("IPAK_PROJECT_NAME", &project_metadata.about.package.name)
        .env(
            "IPAK_PROJECT_VERSION",
            project_metadata.about.package.version.to_string(),
        )
        .env("IPAK_REMOVE_MODE", opts.remove_mode.to_string())
        .arg("ipak/scripts/remove.sh");

    
    let status = remove_process
        .status()
        .map_err(|e| format!("Failed to execute remove process: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        
        Err(format!(
            "Remove process failed with status: {}",
            status.code().unwrap_or(-1) 
        ))
    }
}
