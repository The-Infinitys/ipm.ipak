use crate::modules::version::Version;
use crate::utils::shell;
use crate::{modules::pkg::PackageData, utils::files::file_creation};
use std::str::FromStr;
use std::{
    io::{self, Error, ErrorKind},
    process::Command,
};




struct SetUpItem {
    path: String,
    content: String,
}














fn setup_files(setup_list: Vec<SetUpItem>) -> Result<(), io::Error> {
    for item in setup_list {
        
        file_creation(&item.path, &item.content).map_err(|e| {
            Error::new(
                e.kind(),
                format!("Failed to create file '{}': {}", item.path, e),
            )
        })?;
    }
    Ok(())
}










pub fn default(pkg_data: PackageData) -> Result<PackageData, io::Error> {
    let setup_list = vec![
        SetUpItem {
            path: "ipak/scripts/build.sh".to_string(),
            content: include_str!(
                "templates/default/ipak/scripts/build.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/install.sh".to_string(),
            content: include_str!(
                "templates/default/ipak/scripts/install.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/remove.sh".to_string(),
            content: include_str!(
                "templates/default/ipak/scripts/remove.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/purge.sh".to_string(),
            content: include_str!(
                "templates/default/ipak/scripts/purge.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/README.md".to_string(),
            content: include_str!("templates/script-README.md")
                .to_string(),
        },
    ];
    setup_files(setup_list)?;
    Ok(pkg_data)
}












pub fn rust(pkg_data: PackageData) -> Result<PackageData, io::Error> {
    
    let mut pkg_data = pkg_data;
    pkg_data.about.package.version =
        Version::from_str("0.1.0").map_err(|e| -> io::Error {
            io::Error::new(io::ErrorKind::InvalidInput, e)
        })?;
    if !shell::is_cmd_available("cargo") {
        let rustup_url = "https:
        eprintln!("Error: 'cargo' command not found.");
        eprintln!(
            "To create a Rust project, you need to install Cargo (Rust's package manager)."
        );
        eprintln!(
            "Please visit {} for installation instructions.",
            rustup_url
        );
        return Err(Error::new(
            ErrorKind::NotFound,
            "Cargo command not found. Please install Rust and Cargo.",
        ));
    }

    
    let status =
        Command::new("cargo").arg("init").status().map_err(|e| {
            Error::other(format!("Failed to execute 'cargo init': {}", e))
        })?;

    if !status.success() {
        return Err(Error::other(format!(
            "'cargo init' command failed with exit status: {}",
            status
        )));
    }

    
    let setup_list = vec![
        SetUpItem {
            path: "ipak/scripts/build.sh".to_string(),
            content: include_str!("templates/rust/ipak/scripts/build.sh")
                .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/install.sh".to_string(),
            content: include_str!(
                "templates/rust/ipak/scripts/install.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/remove.sh".to_string(),
            content: include_str!("templates/rust/ipak/scripts/remove.sh")
                .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/purge.sh".to_string(),
            content: include_str!("templates/rust/ipak/scripts/purge.sh")
                .to_string(),
        },
        SetUpItem {
            path: "ipak/project-ignore.yaml".to_string(),
            content: include_str!(
                "templates/rust/ipak/project-ignore.yaml"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/README.md".to_string(),
            content: include_str!("templates/script-README.md")
                .to_string(),
        },
    ];
    setup_files(setup_list)?;
    Ok(pkg_data)
}












pub fn python(pkg_data: PackageData) -> Result<PackageData, io::Error> {
    if !shell::is_cmd_available("python3") {
        let python_url = "https:
        eprintln!("Error: 'python3' command not found.");
        eprintln!(
            "To create a Python project, you need to install Python 3."
        );
        eprintln!(
            "Please visit {} for installation instructions.",
            python_url
        );
        return Err(Error::new(
            ErrorKind::NotFound,
            "python3 command not found. Please install Python 3.",
        ));
    }

    
    
    let venv_status = Command::new("python3")
        .args(["-m", "venv", "venv"]) 
        .status()
        .map_err(|e| {
            Error::other(format!(
                "Failed to execute 'python3 -m venv venv': {}",
                e
            ))
        })?;

    if !venv_status.success() {
        return Err(Error::other(format!(
            "'python3 -m venv venv' command failed with exit status: {}",
            venv_status
        )));
    }
    eprintln!(
        "Virtual environment 'venv' created successfully in the current directory."
    );

    
    let setup_list = vec![
        
        SetUpItem {
            path: "ipak/scripts/build.sh".to_string(),
            content: include_str!(
                "templates/python/ipak/scripts/build.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/install.sh".to_string(),
            content: include_str!(
                "templates/python/ipak/scripts/install.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/remove.sh".to_string(),
            content: include_str!(
                "templates/python/ipak/scripts/remove.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/purge.sh".to_string(),
            content: include_str!(
                "templates/python/ipak/scripts/purge.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/README.md".to_string(),
            content: include_str!("templates/script-README.md")
                .to_string(), 
        },
        SetUpItem {
            path: format!("{}/__main__.py", &pkg_data.about.package.name),
            content: include_str!("templates/python/src/__main__.py")
                .to_string(),
        },
        SetUpItem {
            path: format!("{}/__init__.py", &pkg_data.about.package.name),
            content: include_str!("templates/python/src/__init__.py")
                .to_string(),
        },
        SetUpItem {
            path: "pyproject.toml".to_string(),
            content: include_str!("templates/python/pyproject.toml")
                .to_string()
                .replace("project-name", &pkg_data.about.package.name),
        },
        SetUpItem {
            path: "ipak/project-ignore.yaml".to_string(),
            content: include_str!(
                "templates/python/ipak/project-ignore.yaml"
            )
            .to_string(),
        },
    ];
    setup_files(setup_list)?;
    Ok(pkg_data)
}

pub fn dotnet(pkg_data: PackageData) -> Result<PackageData, io::Error> {
    
    let mut pkg_data = pkg_data;
    if !shell::is_cmd_available("dotnet") {
        let dotnet_url = "https:
        eprintln!("Error: 'dotnet' command not found.");
        eprintln!("To create a .NET project, you need to install .NET");
        eprintln!("For more information, please visit {}.", dotnet_url);
        return Err(Error::new(
            ErrorKind::NotFound,
            "dotnet command not found. Please install .NET.",
        ));
    }
    pkg_data.relation.depend_cmds.push("dotnet".to_owned());
    
    let status = Command::new("dotnet")
        .arg("new")
        .arg("console")
        .arg("--output=./")
        .status()
        .map_err(|e| {
            Error::other(format!("Failed to execute 'dotnet new': {}", e))
        })?;

    if !status.success() {
        return Err(Error::other(format!(
            "'dotnet new' command failed with exit status: {}",
            status
        )));
    }

    
    let setup_list = vec![
        SetUpItem {
            path: "ipak/scripts/build.sh".to_string(),
            content: include_str!(
                "templates/dotnet/ipak/scripts/build.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/install.sh".to_string(),
            content: include_str!(
                "templates/dotnet/ipak/scripts/install.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/remove.sh".to_string(),
            content: include_str!(
                "templates/dotnet/ipak/scripts/remove.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/purge.sh".to_string(),
            content: include_str!(
                "templates/dotnet/ipak/scripts/purge.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/project-ignore.yaml".to_string(),
            content: include_str!(
                "templates/dotnet/ipak/project-ignore.yaml"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/README.md".to_string(),
            content: include_str!("templates/script-README.md")
                .to_string(),
        },
    ];
    setup_files(setup_list)?;
    Ok(pkg_data)
}

pub fn clang(pkg_data: PackageData) -> Result<PackageData, io::Error> {
    
    let mut pkg_data = pkg_data;
    if !shell::is_cmd_available("cmake") {
        let clang_url = "https:
        eprintln!("Error: 'clang' command not found.");
        eprintln!("To create a C++ project, you need to install clang");
        eprintln!("For more information, please visit {}.", clang_url);
        return Err(Error::new(
            ErrorKind::NotFound,
            "clang command not found. Please install clang.",
        ));
    }
    pkg_data.relation.depend_cmds.push("cmake".to_owned());
    
    let setup_list = vec![
        SetUpItem {
            path: "ipak/scripts/build.sh".to_string(),
            content: include_str!("templates/clang/ipak/scripts/build.sh")
                .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/install.sh".to_string(),
            content: include_str!(
                "templates/clang/ipak/scripts/install.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/remove.sh".to_string(),
            content: include_str!(
                "templates/clang/ipak/scripts/remove.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/purge.sh".to_string(),
            content: include_str!("templates/clang/ipak/scripts/purge.sh")
                .to_string(),
        },
        SetUpItem {
            path: "ipak/project-ignore.yaml".to_string(),
            content: include_str!(
                "templates/clang/ipak/project-ignore.yaml"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/README.md".to_string(),
            content: include_str!("templates/script-README.md")
                .to_string(),
        },
        SetUpItem {
            path: "src/main.cpp".to_string(),
            content: include_str!("templates/clang/src/main.cpp")
                .to_string(),
        },
        SetUpItem {
            path: "CMakeLists.txt".to_string(),
            content: include_str!("templates/clang/CMakeLists.txt")
                .to_string()
                .replace("{name}", &pkg_data.about.package.name),
        },
    ];
    setup_files(setup_list)?;
    Ok(pkg_data)
}
