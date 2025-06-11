#!/bin/bash
# install_deps_from_source.sh

# エラーが発生した場合、スクリプトを停止する
set -e

# --- 設定 ---
# 依存関係のソースコードをダウンロードするディレクトリ
DOWNLOAD_DIR=$(pwd)/lib/deps/src_deps
# ビルドされた依存関係をインストールするディレクトリ
INSTALL_DIR=$(pwd)/lib/deps/opt_deps

# ダウンロードおよびインストールディレクトリが存在することを確認し、存在しない場合は作成
mkdir -p "$DOWNLOAD_DIR"
mkdir -p "$INSTALL_DIR"

echo "============================================================"
echo "依存関係のソースコードを '$DOWNLOAD_DIR' にダウンロードし、"
echo "ビルドされたファイルを '$INSTALL_DIR' にインストールします。"
echo "この処理には時間がかかる場合があります。"
echo "============================================================"
echo ""

# --- パッケージビルド用のヘルパー関数 ---
# 引数: name, url, configure_options
build_package() {
    local name="$1"
    local url="$2"
    local config_options="$3"
    local tarball_name=$(basename "$url")
    local extracted_dir=$(basename "$tarball_name" .tar.xz | sed 's/\.tar\.gz$//' | sed 's/\.tgz$//' | sed 's/\.tar\.bz2$//')

    echo "--- $name のビルドを開始 ---"
    echo "ソースダウンロードURL: $url"

    # ダウンロード
    if [ ! -f "$DOWNLOAD_DIR/$tarball_name" ]; then
        echo "ダウンロード中: $tarball_name"
        wget -P "$DOWNLOAD_DIR" "$url" || { echo "エラー: $tarball_name のダウンロードに失敗しました。"; exit 1; }
    else
        echo "すでにダウンロード済み: $tarball_name"
    fi

    # 展開
    echo "展開中: $tarball_name"
    # 以前の展開ディレクトリをクリーンアップ
    rm -rf "$DOWNLOAD_DIR/$extracted_dir"
    tar -xf "$DOWNLOAD_DIR/$tarball_name" -C "$DOWNLOAD_DIR" || { echo "エラー: $tarball_name の展開に失敗しました。"; exit 1; }

    # ビルドディレクトリへ移動
    cd "$DOWNLOAD_DIR/$extracted_dir" || { echo "エラー: $DOWNLOAD_DIR/$extracted_dir ディレクトリへの移動に失敗しました。"; exit 1; }

    # 設定 (configure)
    echo "設定中 ($name): ./configure --prefix=$INSTALL_DIR/$name $config_options"
    ./configure --prefix="$INSTALL_DIR/$name" $config_options || { echo "エラー: $name の configure に失敗しました。"; exit 1; }

    # チェック
    echo "チェック中 ($name)..."
    make check

    # コンパイル
    echo "コンパイル中 ($name)..."
    make -j$(nproc) || { echo "エラー: $name の make に失敗しました。"; exit 1; }

    # インストール
    echo "インストール中 ($name)..."
    make install || { echo "エラー: $name の make install に失敗しました。"; exit 1; }

    echo "--- $name のビルドが完了 ---"
    echo ""
}

# --- 依存関係のビルド順序 (依存関係の解決を考慮) ---

# 1. GMP (MPFR, ISL, GDBが依存)
build_package "gmp" "https://ftp.gnu.org/gnu/gmp/gmp-6.3.0.tar.xz" "--enable-cxx"

# 2. MPFR (GDBが依存, GMPに依存)
build_package "mpfr" "https://www.mpfr.org/mpfr-current/mpfr-4.2.1.tar.xz" "--with-gmp=$INSTALL_DIR/gmp"

# 3. Zlib (Binutils, Elfutilsが依存)
# Zlibのconfigureは標準GNU configureとは少し異なるが、--prefixオプションはサポートされている
build_package "zlib" "https://www.zlib.net/zlib-1.3.1.tar.gz" ""

# 4. Libisl (Binutilsが依存, GMPに依存)
build_package "isl" "https://libisl.sourceforge.io/isl-0.26.tar.xz" "--with-gmp-prefix=$INSTALL_DIR/gmp"

# 5. Elfutils (Libelfを提供, Zlibに依存)
build_package "elfutils" "https://sourceware.org/elfutils/ftp/0.190/elfutils-0.190.tar.bz2" "--with-zlib --enable-libelf-by-default --disable-debuginfod"

# 6. Texinfo (makeinfoを提供, ドキュメント生成に必要)
build_package "texinfo" "https://ftp.gnu.org/gnu/texinfo/texinfo-7.1.tar.gz" ""

# 7. Flex (binutils-gdbのビルドプロセスで利用される可能性)
build_package "flex" "https://github.com/westes/flex/releases/download/v2.6.4/flex-2.6.4.tar.gz" ""

# 8. Bison (binutils-gdbのビルドプロセスで利用される可能性)
build_package "bison" "https://ftp.gnu.org/gnu/bison/bison-3.8.2.tar.xz" ""

echo "============================================================"
echo "すべての依存関係のビルドとインストールが完了しました。"
echo "次にbinutils-gdbのビルドスクリプトを実行できます。"
echo "============================================================"
