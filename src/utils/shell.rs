pub mod question;
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::Stdio;
use std::process::{Command, Output};
use termimad::crossterm::style::{Attribute::*, Color::*};
use termimad::*; 

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
        String::from(username.trim())
    } else if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        let username: String =
            String::from_utf8(output.stdout).unwrap().trim().to_owned();
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
        std::env::var("PAGER").unwrap_or_else(|_| "less".to_string()); 

    let pager_name = {
        let path = std::path::Path::new(&pager_command_str);
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&pager_command_str)
            .to_lowercase()
    };

    let mut command = Command::new(&pager_command_str);

    
    let mut _args_applied = false;
    match pager_name.as_str() {
        "less" => {
            command
                .arg("-R") 
                .arg("-F") 
                .arg("-X") 
                .arg("-K") 
                .arg("-"); 
            _args_applied = true;
        }
        "more" => {
            
            
            
            
            
            
            _args_applied = true; 
        }
        
        
        
        
        
        _ => {
            
            
        }
    }

    
    let mut child_result = command.stdin(Stdio::piped()).spawn();

    
    if let Err(ref e) = child_result {
        eprintln!(
            "Warning: Pager '{}' failed to start with specific arguments ({}). Retrying without arguments.",
            pager_command_str, e
        );
        command = Command::new(&pager_command_str); 
        child_result = command.stdin(Stdio::piped()).spawn();
    }

    let mut child = match child_result {
        Ok(child) => child,
        Err(e) => {
            eprintln!(
                "Error: Pager '{}' failed to start ({}). Printing directly to stdout.",
                pager_command_str, e
            );
            
            io::stdout()
                .write_all(target_string.as_bytes())
                .expect("Failed to write to stdout");
            return;
        }
    };

    
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(target_string.as_bytes()) {
            eprintln!(
                "Error: Failed to write to pager '{}' stdin ({}). Printing directly to stdout.",
                pager_command_str, e
            );
            
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
        
        io::stdout()
            .write_all(target_string.as_bytes())
            .expect("Failed to write to stdout");
        return;
    }

    
    let output = child
        .wait_with_output()
        .expect("failed to wait for pager process");

    if !output.status.success() {
        
        
        if !output.stderr.is_empty() {
            eprintln!(
                "Pager '{}' exited with error: {}",
                pager_command_str,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

pub fn markdown(md_text: String) -> String {
    let mut skin = MadSkin::default();
    
    skin.bold.set_fg(gray(20));
    
    skin.strikeout = CompoundStyle::new(Some(Red), None, Bold.into());
    format!("{}", skin.term_text(&md_text))
}
