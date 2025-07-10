mod global;
mod local;
use crate::utils::shell::is_superuser;
pub fn configure(arg: Option<bool>) -> Result<(), std::io::Error> {
    match arg {
        Some(is_local) => {
            if is_local {
                local::configure()
            } else {
                global::configure()
            }
        }
        None => {
            if is_superuser() {
                global::configure()
            } else {
                local::configure()
            }
        }
    }
}
