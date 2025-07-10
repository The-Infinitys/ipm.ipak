use crate::utils::color::colorize::*;
use dialoguer;
use regex::Regex;

/// プロンプトを表示し、ユーザーからの文字列入力を受け取ります。
/// 入力に失敗した場合はパニックします。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
///
/// # Returns
/// ユーザーが入力した文字列
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

/// ユーザーにyes/noの質問をします。
///
/// # Arguments
/// * `msg` - 質問として表示するメッセージ
///
/// # Returns
/// `Ok(true)`: "yes"または"y"と入力された場合
/// `Ok(false)`: "no"または"n"と入力された場合
/// `Err(String)`: 無効な回答が入力された場合
pub fn yesno(msg: &str) -> Result<bool, String> {
    let input = str_input(msg).trim().to_lowercase();
    let s = input.as_str();
    match s {
        "yes" | "y" => Ok(true),
        "no" | "n" => Ok(false),
        _ => Err(format!("無効な回答: {}", s)),
    }
}

/// ユーザーにyes/noの質問を繰り返し行い、有効な回答が得られるまで続けます。
///
/// # Arguments
/// * `msg` - 質問として表示するメッセージ
///
/// # Returns
/// `true`: "yes"または"y"と入力された場合
/// `false`: "no"または"n"と入力された場合
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

/// プロンプトを表示し、正規表現に一致する文字列入力を受け取ります。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
/// * `regex` - 入力を検証するための正規表現
///
/// # Returns
/// `Ok(String)`: 正規表現に一致する文字列が入力された場合
/// `Err(String)`: 正規表現に一致しない入力がされた場合
pub fn regex_string(msg: &str, regex: Regex) -> Result<String, String> {
    let input = str_input(msg).trim().to_string();
    match regex.is_match(&input) {
        true => Ok(input),
        false => Err(format!("無効な入力: {}", input)),
    }
}

/// プロンプトを表示し、camelCase形式の文字列入力を受け取ります。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
///
/// # Returns
/// `Ok(String)`: camelCase形式の文字列が入力された場合
/// `Err(String)`: 無効な入力がされた場合
pub fn camel_case(msg: &str) -> Result<String, String> {
    let camel_regex =
        Regex::new(r"^[a-z][a-z0-9]*(?:[A-Z][a-z0-9]*)*$").unwrap();
    regex_string(msg, camel_regex)
}

/// ユーザーにcamelCase形式の文字列入力を繰り返し求め、有効な回答が得られるまで続けます。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
///
/// # Returns
/// camelCase形式の文字列
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

/// プロンプトを表示し、PascalCase形式の文字列入力を受け取ります。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
///
/// # Returns
/// `Ok(String)`: PascalCase形式の文字列が入力された場合
/// `Err(String)`: 無効な入力がされた場合
pub fn pascal_case(msg: &str) -> Result<String, String> {
    let pascal_regex =
        Regex::new(r"^[A-Z][a-z0-9]*(?:[A-Z][a-z0-9]*)*$").unwrap();
    regex_string(msg, pascal_regex)
}

/// ユーザーにPascalCase形式の文字列入力を繰り返し求め、有効な回答が得られるまで続けます。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
///
/// # Returns
/// PascalCase形式の文字列
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

/// プロンプトを表示し、snake_case形式の文字列入力を受け取ります。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
///
/// # Returns
/// `Ok(String)`: snake_case形式の文字列が入力された場合
/// `Err(String)`: 無効な入力がされた場合
pub fn snake_case(msg: &str) -> Result<String, String> {
    let snake_regex = Regex::new(r"^[a-z0-9]+(?:_[a-z0-9]+)*$").unwrap();
    regex_string(msg, snake_regex)
}

/// ユーザーにsnake_case形式の文字列入力を繰り返し求め、有効な回答が得られるまで続けます。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
///
/// # Returns
/// snake_case形式の文字列
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

/// プロンプトを表示し、kebab-case形式の文字列入力を受け取ります。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
///
/// # Returns
/// `Ok(String)`: kebab-case形式の文字列が入力された場合
/// `Err(String)`: 無効な入力がされた場合
pub fn kebab_case(msg: &str) -> Result<String, String> {
    let kebab_regex = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    regex_string(msg, kebab_regex)
}

/// ユーザーにkebab-case形式の文字列入力を繰り返し求め、有効な回答が得られるまで続けます。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
///
/// # Returns
/// kebab-case形式の文字列
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

/// プロンプトを表示し、SCREAMING_SNAKE_CASE形式の文字列入力を受け取ります。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
///
/// # Returns
/// `Ok(String)`: SCREAMING_SNAKE_CASE形式の文字列が入力された場合
/// `Err(String)`: 無効な入力がされた場合
pub fn screaming_snake_case(msg: &str) -> Result<String, String> {
    let screaming_snake_regex =
        Regex::new(r"^[A-Z0-9]+(?:_[A-Z0-9]+)*$").unwrap();
    regex_string(msg, screaming_snake_regex)
}

/// ユーザーにSCREAMING_SNAKE_CASE形式の文字列入力を繰り返し求め、有効な回答が得られるまで続けます。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
///
/// # Returns
/// SCREAMING_SNAKE_CASE形式の文字列
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

/// プロンプトを表示し、メールアドレス形式の文字列入力を受け取ります。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
///
/// # Returns
/// `Ok(String)`: メールアドレス形式の文字列が入力された場合
/// `Err(String)`: 無効な入力がされた場合
pub fn email_address(msg: &str) -> Result<String, String> {
    let email_regex =
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .unwrap();
    regex_string(msg, email_regex)
}

/// ユーザーにメールアドレス形式の文字列入力を繰り返し求め、有効な回答が得られるまで続けます。
///
/// # Arguments
/// * `msg` - プロンプトとして表示するメッセージ
///
/// # Returns
/// メールアドレス形式の文字列
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

/// ユーザーに選択肢を提示し、選択された項目を返します。
///
/// # Arguments
/// * `msg` - 選択肢のプロンプトとして表示するメッセージ
/// * `options` - ユーザーに提示する選択肢の文字列スライス
///
/// # Returns
/// ユーザーが選択した項目の文字列
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
