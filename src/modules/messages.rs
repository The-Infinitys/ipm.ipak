use crate::utils::shell::{self, markdown};

struct CargoPackageInfo {
    name: &'static str,

    version: &'static str,

    architecture: &'static str,
}

fn get_info() -> CargoPackageInfo {
    CargoPackageInfo {
        name: option_env!("CARGO_PKG_NAME").unwrap_or("ipak"),
        version: option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
        architecture: std::env::consts::ARCH,
    }
}

fn insert_info(text: &'static str) -> String {
    let cargo_package = get_info();
    let replace_list = vec![
        ["name", cargo_package.name],
        ["version", cargo_package.version],
        ["architecture", cargo_package.architecture],
    ];
    let mut text = text.to_string();
    for replaces in replace_list {
        text = text
            .replace(format!("{{{}}}", replaces[0]).as_str(), replaces[1]);
    }
    text
}

pub fn manual() -> Result<(), std::io::Error> {
    let manual_str =
        markdown(insert_info(include_str!("./messages/manual.md")));
    shell::pager(manual_str.to_string());
    Ok(())
}
