/// デバッグビルド時のみ、標準出力に改行付きで出力するマクロ。
///
/// リリースビルド時にはコンパイル時に削除され、何も行いません。
/// 標準ライブラリの `println!` と同様の使い方ができます。
#[macro_export] // クレート外から利用可能にする
macro_rules! dprintln {
    // 引数なしのprintln!に相当
    // 標準のprintln!と同様に、フォーマット文字列と引数を受け取る
    ($($arg:tt)*) => ({
        #[cfg(debug_assertions)] // デバッグビルド時のみ有効
        println!($($arg)*);
    });
}

/// リリースビルド時（または debug_assertions が無効な場合）には、dprintln! は何もしない。
/// #[cfg(debug_assertions)] の定義のみで十分ですが、明示的に定義することも可能です（通常は不要）。
/// #[cfg(not(debug_assertions))]
/// #[macro_export]
/// macro_rules! dprintln {
///     ($($arg:tt)*) => {}; // 何もしない
/// }
///
/// デバッグビルド時のみ、標準出力に改行なしで出力するマクロ。
///
/// リリースビルド時にはコンパイル時に削除され、何も行いません。
/// 標準ライブラリの `print!` と同様の使い方ができます。
#[macro_export] // クレート外から利用可能にする
macro_rules! dprint {
    // 標準のprint!と同様に、フォーマット文字列と引数を受け取る
    ($($arg:tt)*) => ({
        #[cfg(debug_assertions)] // デバッグビルド時のみ有効
        print!($($arg)*);
    });
}
