pub mod archive;
pub mod color;
pub mod debug;
pub mod files;
pub mod shell;
pub mod error;
pub mod args;
pub fn generate_email_address() -> String {
    let username = shell::username();
    let hostname = shell::hostname();
    let domain = "local";
    format!("{}@{}.{}", username, hostname, domain)
}
