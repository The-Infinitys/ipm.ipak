use std::path::PathBuf;
fn root_path() -> PathBuf {
    PathBuf::from("/")
}
fn ipak_path() -> PathBuf {
    root_path().join("etc/ipak/")
}
pub fn packageslist_filepath() -> PathBuf {
    packages_dirpath().join("list.yaml")
}
pub fn packages_dirpath() -> PathBuf {
    ipak_path().join("packages")
}
