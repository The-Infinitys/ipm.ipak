use std::{
    env,
    io::{Error, Write},
    path,
};

/// 指定されたパスにディレクトリを作成します。既に存在する場合は何もしません。
/// 親ディレクトリが存在しない場合は、それらも作成します。
///
/// # Arguments
/// * `path_str` - 作成するディレクトリのパスを表す文字列スライス
///
/// # Returns
/// `Ok(())`: ディレクトリが正常に作成されたか、既に存在していた場合
/// `Err(IpakError)`: ディレクトリの作成に失敗した場合
pub fn dir_creation(path_str: &str) -> Result<(), Error> {
    let path = path::Path::new(path_str);
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// 指定されたパスにファイルを作成し、コンテンツを書き込みます。
/// ファイルの親ディレクトリが存在しない場合は、それらも作成します。
///
/// # Arguments
/// * `path_str` - 作成するファイルのパスを表す文字列スライス
/// * `content` - ファイルに書き込むコンテンツを表す文字列スライス
///
/// # Returns
/// `Ok(())`: ファイルが正常に作成され、コンテンツが書き込まれた場合
/// `Err(IpakError)`: ファイルの作成または書き込みに失敗した場合
pub fn file_creation(path_str: &str, content: &str) -> Result<(), Error> {
    let path = path::Path::new(path_str);

    if let Some(parent_dir) = path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }

    let mut file = std::fs::File::create(path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

/// 指定されたパスのファイルまたはディレクトリが存在するかどうかを確認します。
/// パスは現在の作業ディレクトリからの相対パスとして扱われます。
///
/// # Arguments
/// * `path_str` - 存在を確認するファイルまたはディレクトリのパスを表す文字列スライス
///
/// # Returns
/// `true`: ファイルまたはディレクトリが存在する場合
/// `false`: ファイルまたはディレクトリが存在しない場合
pub fn is_exists(path_str: &str) -> bool {
    env::current_dir().unwrap().join(path_str).exists()
}

/// 指定されたパスのファイルが存在するかどうかを確認します。
/// パスは現在の作業ディレクトリからの相対パスとして扱われます。
///
/// # Arguments
/// * `path_str` - 存在を確認するファイルのパスを表す文字列スライス
///
/// # Returns
/// `true`: ファイルが存在する場合
/// `false`: ファイルが存在しないか、ディレクトリである場合
pub fn is_file_exists(path_str: &str) -> bool {
    env::current_dir().unwrap().join(path_str).is_file()
}

/// 指定されたパスのディレクトリが存在するかどうかを確認します。
/// パスは現在の作業ディレクトリからの相対パスとして扱われます。
///
/// # Arguments
/// * `path_str` - 存在を確認するディレクトリのパスを表す文字列スライス
///
/// # Returns
/// `true`: ディレクトリが存在する場合
/// `false`: ディレクトリが存在しないか、ファイルである場合
pub fn is_dir_exists(path_str: &str) -> bool {
    env::current_dir().unwrap().join(path_str).is_dir()
}
