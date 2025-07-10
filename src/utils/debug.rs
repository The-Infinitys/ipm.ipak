/// デバッグビルドでのみ動作する`println!`マクロです。
/// `debug_assertions`が有効な場合にのみ、指定された引数を標準出力に出力します。
/// リリースビルドでは何もしません。
#[macro_export]
macro_rules! dprintln {
    ($($arg:tt)*) => ({
        #[cfg(debug_assertions)]
        println!($($arg)*);
    });
}

/// デバッグビルドでのみ動作する`print!`マクロです。
/// `debug_assertions`が有効な場合にのみ、指定された引数を標準出力に出力します。
/// リリースビルドでは何もしません。
#[macro_export]
macro_rules! dprint {
    ($($arg:tt)*) => ({
        #[cfg(debug_assertions)]
        print!($($arg)*);
    });
}
