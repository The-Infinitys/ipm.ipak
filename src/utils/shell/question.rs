use crate::utils::color::colorize::*;
use dialoguer;
use regex::Regex;










fn str_input(msg: &str) -> String {
    dialoguer::Input::with_theme(
        &dialoguer::theme::ColorfulTheme::default(),
    )
    .with_prompt(msg)
    .interact_text()
    .unwrap_or_else(|_| {
        panic!("正しい文字列が入力されませんでした");
    })
}












pub fn yesno(msg: &str) -> Result<bool, String> {
    let input = str_input(msg).trim().to_lowercase();
    let s = input.as_str();
    match s {
        "yes" | "y" => Ok(true),
        "no" | "n" => Ok(false),
        _ => Err(format!("無効な回答: {}", s)),
    }
}










pub fn yesno_loop(msg: &str) -> bool {
    loop {
        match yesno(msg) {
            Ok(answer) => return answer,
            Err(error) => {
                print!("({}) ", error.red());
                continue;
            }
        };
    }
}












pub fn regex_string(msg: &str, regex: Regex) -> Result<String, String> {
    let input = str_input(msg).trim().to_string();
    match regex.is_match(&input) {
        true => Ok(input),
        false => Err(format!("無効な入力: {}", input)),
    }
}











pub fn camel_case(msg: &str) -> Result<String, String> {
    let camel_regex =
        Regex::new(r"^[a-z][a-z0-9]*(?:[A-Z][a-z0-9]*)*$").unwrap();
    regex_string(msg, camel_regex)
}










pub fn camel_loop(msg: &str) -> String {
    loop {
        match camel_case(msg) {
            Ok(answer) => return answer,
            Err(error) => {
                print!("({}) ", error.red());
                continue;
            }
        };
    }
}











pub fn pascal_case(msg: &str) -> Result<String, String> {
    let pascal_regex =
        Regex::new(r"^[A-Z][a-z0-9]*(?:[A-Z][a-z0-9]*)*$").unwrap();
    regex_string(msg, pascal_regex)
}










pub fn pascal_loop(msg: &str) -> String {
    loop {
        match pascal_case(msg) {
            Ok(answer) => return answer,
            Err(error) => {
                print!("({}) ", error.red());
                continue;
            }
        };
    }
}











pub fn snake_case(msg: &str) -> Result<String, String> {
    let snake_regex = Regex::new(r"^[a-z0-9]+(?:_[a-z0-9]+)*$").unwrap();
    regex_string(msg, snake_regex)
}










pub fn snake_loop(msg: &str) -> String {
    loop {
        match snake_case(msg) {
            Ok(answer) => return answer,
            Err(error) => {
                print!("({}) ", error.red());
                continue;
            }
        };
    }
}











pub fn kebab_case(msg: &str) -> Result<String, String> {
    let kebab_regex = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    regex_string(msg, kebab_regex)
}










pub fn kebab_loop(msg: &str) -> String {
    loop {
        match kebab_case(msg) {
            Ok(answer) => return answer,
            Err(error) => {
                print!("({}) ", error.red());
                continue;
            }
        };
    }
}











pub fn screaming_snake_case(msg: &str) -> Result<String, String> {
    let screaming_snake_regex =
        Regex::new(r"^[A-Z0-9]+(?:_[A-Z0-9]+)*$").unwrap();
    regex_string(msg, screaming_snake_regex)
}










pub fn screaming_snake_loop(msg: &str) -> String {
    loop {
        match screaming_snake_case(msg) {
            Ok(answer) => return answer,
            Err(error) => {
                print!("({}) ", error.red());
                continue;
            }
        };
    }
}











pub fn email_address(msg: &str) -> Result<String, String> {
    let email_regex =
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .unwrap();
    regex_string(msg, email_regex)
}










pub fn email_loop(msg: &str) -> String {
    loop {
        match email_address(msg) {
            Ok(answer) => return answer,
            Err(error) => {
                print!("({}) ", error.red());
                continue;
            }
        };
    }
}

pub fn select(msg: &str, options: &[&str]) -> String {
    let selection = dialoguer::Select::with_theme(
        &dialoguer::theme::ColorfulTheme::default(),
    )
    .with_prompt(msg.green().to_string())
    .default(0)
    .items(options)
    .interact()
    .unwrap();
    options[selection].to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
