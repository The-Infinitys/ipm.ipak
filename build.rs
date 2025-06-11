use cmake;
fn main() {
    // CMakeLists.txtが存在するディレクトリを指定します
    // lib/cmake-exampleディレクトリからの相対位置となります
    let dst = cmake::build("lib/binutils-gdb");
    println!("cargo:rustc-link-search=native={}", dst.display());

    // lib/binutils-gdb からビルドされた静的ライブラリをリンクします。
    // "your_binutils_gdb_library_name" の部分は、
    // lib/binutils-gdb/CMakeLists.txt 内の add_library() コマンドで定義されている
    // 実際のターゲットライブラリ名に置き換えてください。
    // 例: add_library(my_gdb_component STATIC ...) の場合は "my_gdb_component" となります。
    println!("cargo:rustc-link-lib=static=your_binutils_gdb_library_name");
    // C++ソースコードの場合は必ずこれを追加すること
    println!("cargo:rustc-link-lib=dylib=stdc++");

    // CMakeLists.txt内の記述とは別に、その他のライブラリは必要なものを全て記述する必要あり
    println!("cargo:rustc-link-lib=dylib=EGL");
    println!("cargo:rustc-link-lib=dylib=GLESv2");
    println!("cargo:rustc-link-lib=dylib=X11");

    // pkg-configでヒットしないライブラリは以下のように直接パス指定が可能
    let soil_lib_dir = "/usr/lib";
    println!("cargo:rustc-link-search={}", soil_lib_dir);
    println!("cargo:rustc-link-lib=dylib=SOIL");
}
