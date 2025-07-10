use crate::utils::shell;
use std::env;
use std::path::PathBuf; 
fn home_path() -> PathBuf {
    let home_path_str = env::var("HOME").unwrap_or_else(|_| {
        
        
        eprintln!("Error: HOME environment variable not set. Attempting to use username.");
        let username = shell::username();
        format!("/home/{}", username)
    });
    PathBuf::from(home_path_str)
}
fn ipak_path() -> PathBuf {
    home_path().join(".ipak")
}
pub fn packageslist_filepath() -> PathBuf {
    packages_dirpath().join("list.yaml")
}
pub fn packages_dirpath() -> PathBuf {
    ipak_path().join("packages")
}
pub fn cache_path() -> PathBuf {
    home_path().join(".cache/ipak/")
}
