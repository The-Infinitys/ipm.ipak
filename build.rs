use std::env;
use std::path::Path;
use std::process::Command;

fn exec_shellscript(
    script_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_path = Path::new(script_path);
    let shell = env::var("SHELL").unwrap_or_else(|_| "sh".to_string());

    let status = Command::new(&shell)
        .arg(script_path)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .status()
        .map_err(|e| -> Box<dyn std::error::Error> {
            Box::new(std::io::Error::other(format!(
                "Failed to execute shell script: {}",
                e
            )))
        })?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Shell script exited with non-zero status code: {}",
                code
            ),
        ))),
        None => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Shell script terminated by signal",
        ))),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    exec_shellscript("scripts/preconfigure.sh")?;
    // exec_shellscript("scripts/build/binutils.sh")?;
    Ok(())
}
