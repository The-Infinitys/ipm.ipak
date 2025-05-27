pub mod question;
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::Stdio;
use std::process::{Command, Output};

pub fn is_cmd_available(cmd: &str) -> bool {
    let path_env = env::var("PATH");
    match path_env {
        Ok(path_env) => {
            let check_paths = path_env.split(":");
            for check_path in check_paths {
                let check_path = Path::new(check_path).join(cmd);
                if check_path.is_file() {
                    return true;
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
    false
}

pub fn username() -> String {
    let output: Output = Command::new("whoami")
        .output()
        .expect("failed to execute process");

    if cfg!(target_os = "windows") {
        let info: String = String::from_utf8(output.stdout).unwrap();
        let username: &str = info.split("\\").collect::<Vec<&str>>()[1];
        String::from(username)
    } else if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        let username: String = String::from_utf8(output.stdout).unwrap();
        username
    } else {
        panic!("Error");
    }
}
pub fn hostname() -> String {
    let output: Output = Command::new("hostname")
        .output()
        .expect("failed to execute process");
    let hostname: String =
        String::from_utf8(output.stdout).unwrap().trim().to_owned();
    hostname
}

pub fn shell_type() -> String {
    Path::new(&env::var("SHELL").unwrap_or("unknown".to_string()))
        .file_name()
        .unwrap()
        .to_owned()
        .into_string()
        .unwrap()
}
pub fn is_superuser() -> bool {
    if cfg!(target_os = "windows") {
        return false;
    }
    let output: Output =
        Command::new("id").output().expect("failed to execute process");
    let id: String = String::from_utf8(output.stdout).unwrap();
    id.contains("uid=0(root)")
}
pub fn pager(target_string: String) {
    let pager_command_str =
        std::env::var("PAGER").unwrap_or_else(|_| "less".to_string()); // Default to "less"

    let pager_name = {
        let path = std::path::Path::new(&pager_command_str);
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&pager_command_str)
            .to_lowercase()
    };

    let mut command = Command::new(&pager_command_str);

    // Apply arguments based on the detected pager
    let mut _args_applied = false;
    match pager_name.as_str() {
        "less" => {
            command
                .arg("-R") // Raw control characters (for colored output)
                .arg("-F") // Quit if output fits on one screen
                .arg("-X") // Don't clear the screen when less quits
                .arg("-K") // Exit less directly without prompting if a signal is received
                .arg("-"); // Read input from stdin
            _args_applied = true;
        }
        "more" => {
            // 'more' typically doesn't need specific arguments for piping stdin,
            // but '-R' for raw output (colors) can sometimes be useful if supported.
            // However, it's less consistently supported than in 'less'.
            // For simplicity, we'll just pipe stdin without extra args for 'more' by default.
            // If you want to add an arg like -R for more, you could:
            // command.arg("-R");
            _args_applied = true; // Mark as handled even if no specific args were added
        }
        // Add more pagers here if needed, e.g., "bat"
        // "bat" => {
        //     command.arg("-P"); // --plain, equivalent to piping for bat
        //     args_applied = true;
        // }
        _ => {
            // For unknown pagers, we'll try with no specific arguments first.
            // No arguments added here, so args_applied remains false.
        }
    }

    // Try to spawn the pager with the chosen arguments
    let mut child_result = command.stdin(Stdio::piped()).spawn();

    // If initial spawn fails (e.g., specific args not supported), try again without args
    if let Err(ref e) = child_result {
        eprintln!(
            "Warning: Pager '{}' failed to start with specific arguments ({}). Retrying without arguments.",
            pager_command_str, e
        );
        command = Command::new(&pager_command_str); // Recreate command without specific args
        child_result = command.stdin(Stdio::piped()).spawn();
    }

    let mut child = match child_result {
        Ok(child) => child,
        Err(e) => {
            eprintln!(
                "Error: Pager '{}' failed to start ({}). Printing directly to stdout.",
                pager_command_str, e
            );
            // Fallback: print directly if pager cannot be started
            io::stdout()
                .write_all(target_string.as_bytes())
                .expect("Failed to write to stdout");
            return;
        }
    };

    // Write the target_string to the pager's stdin
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(target_string.as_bytes()) {
            eprintln!(
                "Error: Failed to write to pager '{}' stdin ({}). Printing directly to stdout.",
                pager_command_str, e
            );
            // Fallback: print directly if writing to stdin fails
            io::stdout()
                .write_all(target_string.as_bytes())
                .expect("Failed to write to stdout");
            return;
        }
    } else {
        eprintln!(
            "Error: Failed to open pager '{}' stdin. Printing directly to stdout.",
            pager_command_str
        );
        // Fallback: print directly if stdin is not available
        io::stdout()
            .write_all(target_string.as_bytes())
            .expect("Failed to write to stdout");
        return;
    }

    // Wait for the pager process to finish
    let output = child
        .wait_with_output()
        .expect("failed to wait for pager process");

    if !output.status.success() {
        // Pager exited with a non-zero status. This can happen if the user quits early.
        // Only print stderr if there's actual error output from the pager.
        if !output.stderr.is_empty() {
            eprintln!(
                "Pager '{}' exited with error: {}",
                pager_command_str,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
