use crate::utils::color::colorize::*;
use dialoguer;
use regex::Regex;

/// ユーザーにメッセージを表示し、標準入力から1行の入力を取得します。
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
///
/// # 戻り値
///
/// * `String` - 標準入力から読み取った入力行。
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

/// はい/いいえの質問を行い、ユーザーの回答を検証します。
///
/// # 引数
///
/// * `msg` - ユーザーに尋ねる質問。
///
/// # 戻り値
///
/// * `Ok(true)` - "yes" または "y" が入力された場合。
/// * `Ok(false)` - "no" または "n" が入力された場合。
/// * `Err(String)` - 無効な回答が入力された場合、エラーメッセージを含む。
pub fn yesno(msg: &str) -> Result<bool, String> {
    let input = str_input(msg).trim().to_lowercase();
    let s = input.as_str();
    match s {
        "yes" | "y" => Ok(true),
        "no" | "n" => Ok(false),
        _ => Err(format!("無効な回答: {}", s)),
    }
}

/// 有効なはい/いいえの回答が得られるまで質問を繰り返します。
///
/// # 引数
///
/// * `msg` - ユーザーに尋ねる質問。
///
/// # 戻り値
///
/// * `bool` - "yes" または "y" の場合は true、"no" または "n" の場合は false。
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

/// 指定された正規表現に一致する入力を検証します。
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
/// * `regex` - 入力が一致する必要がある正規表現。
///
/// # 戻り値
///
/// * `Ok(String)` - 入力が正規表現に一致する場合。
/// * `Err(String)` - 入力が一致しない場合、エラーメッセージを含む。
pub fn regex_string(msg: &str, regex: Regex) -> Result<String, String> {
    let input = str_input(msg).trim().to_string();
    match regex.is_match(&input) {
        true => Ok(input),
        false => Err(format!("無効な入力: {}", input)),
    }
}

/// キャメルケース形式の文字列を検証します（例: helloWorld）。
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
///
/// # 戻り値
///
/// * `Ok(String)` - 入力が有効なキャメルケースの場合。
/// * `Err(String)` - 入力が無効な場合、エラーメッセージを含む。
pub fn camel_case(msg: &str) -> Result<String, String> {
    let camel_regex =
        Regex::new(r"^[a-z][a-z0-9]*(?:[A-Z][a-z0-9]*)*$").unwrap();
    regex_string(msg, camel_regex)
}

/// 有効なキャメルケース文字列が得られるまで入力を繰り返します。
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
///
/// # 戻り値
///
/// * `String` - ユーザーが提供した有効なキャメルケース文字列。
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

/// パスカルケース形式の文字列を検証します（例: HelloWorld）。
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
///
/// # 戻り値
///
/// * `Ok(String)` - 入力が有効なパスカルケースの場合。
/// * `Err(String)` - 入力が無効な場合、エラーメッセージを含む。
pub fn pascal_case(msg: &str) -> Result<String, String> {
    let pascal_regex =
        Regex::new(r"^[A-Z][a-z0-9]*(?:[A-Z][a-z0-9]*)*$").unwrap();
    regex_string(msg, pascal_regex)
}

/// 有効なパスカルケース文字列が得られるまで入力を繰り返します。
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
///
/// # 戻り値
///
/// * `String` - ユーザーが提供した有効なパスカルケース文字列。
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

/// スネークケース形式の文字列を検証します（例: hello_world）。
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
///
/// # 戻り値
///
/// * `Ok(String)` - 入力が有効なスネークケースの場合。
/// * `Err(String)` - 入力が無効な場合、エラーメッセージを含む。
pub fn snake_case(msg: &str) -> Result<String, String> {
    let snake_regex = Regex::new(r"^[a-z0-9]+(?:_[a-z0-9]+)*$").unwrap();
    regex_string(msg, snake_regex)
}

/// 有効なスネークケース文字列が得られるまで入力を繰り返します。
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
///
/// # 戻り値
///
/// * `String` - ユーザーが提供した有効なスネークケース文字列。
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

/// ケバブケース形式の文字列を検証します（例: hello-world）。
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
///
/// # 戻り値
///
/// * `Ok(String)` - 入力が有効なケバブケースの場合。
/// * `Err(String)` - 入力が無効な場合、エラーメッセージを含む。
pub fn kebab_case(msg: &str) -> Result<String, String> {
    let kebab_regex = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    regex_string(msg, kebab_regex)
}

/// 有効なケバブケース文字列が得られるまで入力を繰り返します。
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
///
/// # 戻り値
///
/// * `String` - ユーザーが提供した有効なケバブケース文字列。
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

/// SCREAMING_SNAKE_CASE形式の文字列を検証します（例: HELLO_WORLD）。
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
///
/// # 戻り値
///
/// * `Ok(String)` - 入力が有効なSCREAMING_SNAKE_CASEの場合。
/// * `Err(String)` - 入力が無効な場合、エラーメッセージを含む。
pub fn screaming_snake_case(msg: &str) -> Result<String, String> {
    let screaming_snake_regex =
        Regex::new(r"^[A-Z0-9]+(?:_[A-Z0-9]+)*$").unwrap();
    regex_string(msg, screaming_snake_regex)
}

/// 有効なSCREAMING_SNAKE_CASE文字列が得られるまで入力を繰り返します。
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
///
/// # 戻り値
///
/// * `String` - ユーザーが提供した有効なSCREAMING_SNAKE_CASE文字列。
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

/// メールアドレス文字列を検証します (例: user@example.com)
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
///
/// # 戻り値
///
/// * `Ok(String)` - 入力が有効なSCREAMING_SNAKE_CASEの場合。
/// * `Err(String)` - 入力が無効な場合、エラーメッセージを含む。
pub fn email_address(msg: &str) -> Result<String, String> {
    let email_regex =
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .unwrap();
    regex_string(msg, email_regex)
}

/// 有効なメールアドレスが得られるまで入力を繰り返します。
///
/// # 引数
///
/// * `msg` - ユーザーに表示するメッセージ。
///
/// # 戻り値
///
/// * `String` - ユーザーが提供した有効なメールアドレスの文字列。
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
