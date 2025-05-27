pub mod debug;
pub mod files;
pub mod shell;

pub fn generate_email_address() -> String {
    let username = shell::username();
    let hostname = shell::hostname();
    let domain = "local";
    format!("{}@{}.{}", username, hostname, domain)
}
