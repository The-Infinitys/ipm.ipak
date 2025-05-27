use crate::utils::files::file_creation;
use std::env;
use std::io::{Error, ErrorKind};
use std::path::Path;
pub fn configure() -> Result<(), Error> {
    let configure_list =
        [["/etc/ipak/README.md", include_str!("data/global/README.md")]];
    let home_dir = env::var("HOME").unwrap();
    let home_dir = Path::new(&home_dir);
    for configure_data in configure_list {
        let creation_result = file_creation(
            home_dir.join(configure_data[0]).to_str().unwrap(),
            configure_data[1],
        );
        match creation_result {
            Ok(()) => {
                continue;
            }
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}
