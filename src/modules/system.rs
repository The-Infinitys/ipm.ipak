mod configure;
pub mod path;
use crate::utils::{args::SystemCommands, error::Error};
pub fn system(args: SystemCommands) -> Result<(), Error> {
    match args {
        SystemCommands::Configure { local, global } => {
            configure::configure({
                if local && !global {
                    Some(true)
                } else if global && !local {
                    Some(false)
                } else {
                    None
                }
            })
            .map_err(|e| -> Error { e.into() })?
        }
    }
    Ok(())
}
