use super::ExecMode;
use super::ExecShell;
use super::metadata; 
use crate::dprintln;
use crate::utils::color::colorize::*;
use std::fmt::{self, Display};

#[derive(Default)]
pub struct PurgeOptions {
    
    pub purge_shell: ExecShell,
    pub purge_mode: ExecMode,
}

impl Display for PurgeOptions {
    
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", "Purge Options".cyan().bold())?;
        writeln!(
            f,
            "  {}: {}",
            "purge-shell".green().bold(),
            self.purge_shell
        )?;
        writeln!(
            f,
            "  {}: {}",
            "purge-mode".green().bold(),
            self.purge_mode
        )?;
        Ok(())
    }
}














pub fn purge(opts: PurgeOptions) -> Result<(), String> {
    dprintln!("{}", &opts); 

    
    let target_dir = metadata::get_dir().map_err(|_| {
        "Error: Couldn't find Ipak Directory. Make sure you are in an ipak project.".to_string()
    })?;

    
    let project_metadata = metadata::metadata().map_err(|e| {
        format!("Error: Failed to read project metadata: {:?}", e)
    })?;

    
    let mut purge_process = opts.purge_shell.generate();
    purge_process
        .current_dir(&target_dir) 
        .env("IPAK_PROJECT_NAME", &project_metadata.about.package.name) 
        .env(
            "IPAK_PROJECT_VERSION",
            project_metadata.about.package.version.to_string(),
        )
        .env("IPAK_PURGE_MODE", opts.purge_mode.to_string()) 
        .arg("ipak/scripts/purge.sh"); 

    
    let status = purge_process
        .status()
        .map_err(|e| format!("Failed to execute purge process: {}", e))?;

    
    if status.success() {
        Ok(())
    } else {
        
        let code_info = status
            .code()
            .map_or("".to_string(), |c| format!(" (exit code: {})", c));
        Err(format!(
            "Purge process failed with status: {}{}",
            status, code_info
        ))
    }
}
