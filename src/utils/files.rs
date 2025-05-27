use std::{
    env,
    io::{Error, Write},
    path,
};

/// 指定されたパスにディレクトリを作成します。
///
/// 必要に応じて、指定されたパスの親ディレクトリも自動的に作成します。
///
/// # Arguments
///
/// * `path_str` - 作成するディレクトリのパス文字列。
///
/// # Returns
///
/// ディレクトリの作成に成功した場合は `Ok(())`、失敗した場合は `std::io::Error` を返します。
pub fn dir_creation(path_str: &str) -> Result<(), Error> {
    let path = path::Path::new(path_str);
    // std::fs::create_dir_all は指定されたパスの全ての親ディレクトリを含めて作成します。
    // ディレクトリが既に存在してもエラーにはなりません。
    std::fs::create_dir_all(path)?;

    Ok(())
}

/// 指定されたパスにファイルを作成し、内容を書き込みます。
///
/// ファイルの親ディレクトリが存在しない場合は自動的に作成します。
///
/// # Arguments
///
/// * `path_str` - 作成するファイルのパス文字列。
/// * `content` - ファイルに書き込む内容文字列。
///
/// # Returns
///
/// ファイルの作成と書き込みに成功した場合は `Ok(())`、失敗した場合は `std::io::Error` を返します。
pub fn file_creation(path_str: &str, content: &str) -> Result<(), Error> {
    let path = path::Path::new(path_str);

    // ファイルの親ディレクトリを取得し、存在しない場合は作成する
    if let Some(parent_dir) = path.parent() {
        // parent() が Some を返す場合（ルートディレクトリでない場合）のみ処理を実行
        // std::fs::create_dir_all は既に存在するディレクトリに対しては何も行いません
        std::fs::create_dir_all(parent_dir)?;
    }

    // ファイルを作成 (親ディレクトリは既に存在保証されているか、不要な場合)
    // ファイルが既に存在する場合は上書きされます。
    let mut file = std::fs::File::create(path)?;

    // ファイルに内容を書き込む
    file.write_all(content.as_bytes())?;

    Ok(())
}

/// 指定されたパスがカレントディレクトリを基準として存在するかどうかをチェックします。
///
/// # Arguments
///
/// * `path_str` - 存在を確認するパス文字列。
///
/// # Returns
///
/// パスが存在する場合は `true`、存在しない場合は `false` を返します。
pub fn is_exists(path_str: &str) -> bool {
    // カレントディレクトリを取得し、指定されたパスと結合して存在チェックを行います。
    // unwrap() はカレントディレクトリの取得に失敗した場合にパニックします。
    env::current_dir().unwrap().join(path_str).exists()
}

/// 指定されたパスがカレントディレクトリを基準としてファイルとして存在するかどうかをチェックします。
///
/// # Arguments
///
/// * `path_str` - ファイルとして存在するか確認するパス文字列。
///
/// # Returns
///
/// パスがファイルとして存在する場合は `true`、そうでない場合は `false` を返します。
pub fn is_file_exists(path_str: &str) -> bool {
    // カレントディレクトリを取得し、指定されたパスと結合してファイル存在チェックを行います。
    env::current_dir().unwrap().join(path_str).is_file()
}

/// 指定されたパスがカレントディレクトリを基準としてディレクトリとして存在するかどうかをチェックします。
///
/// # Arguments
///
/// * `path_str` - ディレクトリとして存在するか確認するパス文字列。
///
/// # Returns
///
/// パスがディレクトリとして存在する場合は `true`、そうでない場合は `false` を返します。
pub fn is_dir_exists(path_str: &str) -> bool {
    // カレントディレクトリを取得し、指定されたパスと結合してディレクトリ存在チェックを行います。
    env::current_dir().unwrap().join(path_str).is_dir()
}
