#!/bin/bash
# install_deps_from_source.sh

# エラーが発生した場合、スクリプトを停止する
set -e

# --- 設定 ---
# 依存関係のソースコードをダウンロードするディレクトリ
DOWNLOAD_DIR=$(pwd)/lib/deps/src_deps
# ビルドされた依存関係をインストールするディレクトリ
INSTALL_DIR=$(pwd)/lib/deps/opt_deps

# 並列ダウンロードを有効にするフラグ
PARALLEL_DOWNLOAD=false

# --- 引数解析 ---
if [[ "$1" == "--parallel-download" ]]; then
    PARALLEL_DOWNLOAD=true
    echo "--- 並列ダウンロードが有効になりました ---"
    shift # 引数を消費
fi

# ダウンロードおよびインストールディレクトリが存在することを確認し、存在しない場合は作成
mkdir -p "$DOWNLOAD_DIR"
mkdir -p "$INSTALL_DIR"

echo "============================================================"
echo "依存関係のソースコードを '$DOWNLOAD_DIR' にダウンロードし、"
echo "ビルドされたファイルを '$INSTALL_DIR' にインストールします。"
echo "この処理には時間がかかる場合があります。"
echo "============================================================"
echo ""

# --- パッケージ情報定義 ---
# name, url, configure_options の順で配列に格納
# 依存関係の順序を考慮して並び替え
declare -a packages=(
    # ビルドツール/ユーティリティ
    "flex" "https://github.com/westes/flex/releases/download/v2.6.4/flex-2.6.4.tar.gz" ""
    "bison" "https://ftp.gnu.org/gnu/bison/bison-3.8.2.tar.xz" ""
    "texinfo" "https://ftp.gnu.org/gnu/texinfo/texinfo-7.2.tar.gz" ""
    "zlib" "https://zlib.net/fossils/zlib-1.3.1.tar.gz" "" # zlibは多くのツールで利用されるため、早めにビルド

    # 数値計算ライブラリ
    "gmp" "https://ftp.gnu.org/gnu/gmp/gmp-6.3.0.tar.xz" "--disable-shared --enable-fft" # --enable-fft はオプション
    "mpfr" "https://www.mpfr.org/mpfr-current/mpfr-4.2.2.tar.gz" "--disable-shared --with-gmp=$INSTALL_DIR/gmp"
    "isl" "https://libisl.sourceforge.io/isl-0.26.tar.xz" "--disable-shared --with-gmp-prefix=$INSTALL_DIR/gmp --with-mpfr-prefix=$INSTALL_DIR/mpfr"
    
    # その他のライブラリ
    # libarchive を elfutils の前に配置
    "libarchive" "https://www.libarchive.org/downloads/libarchive-3.7.4.tar.gz" "" # 最新の安定版を適宜確認
    "elfutils" "https://sourceware.org/elfutils/ftp/0.190/elfutils-0.190.tar.bz2" "--with-zlib=$INSTALL_DIR/zlib --with-libarchive=$INSTALL_DIR/libarchive"
)

# 展開ディレクトリ情報を保存するための連想配列 (Bash 4.0以上)
declare -A extracted_dirs

# --- ダウンロードフェーズ (並列ダウンロードが有効な場合のみ) ---
if $PARALLEL_DOWNLOAD; then
    echo "--- 全てのソースコードを並列でダウンロード開始 ---"
    pids=()
    for ((i=0; i<${#packages[@]}; i+=3)); do
        name="${packages[$i]}"
        url="${packages[$i+1]}"
        tarball_name=$(basename "$url")

        if [ ! -f "$DOWNLOAD_DIR/$tarball_name" ]; then
            echo "ダウンロード予約: $tarball_name"
            wget -P "$DOWNLOAD_DIR" "$url" &
            pids+=($!)
        else
            echo "すでにダウンロード済み: $tarball_name"
        fi
    done

    # 全てのバックグラウンドダウンロードが完了するのを待つ
    for pid in "${pids[@]}"; do
        wait "$pid"
        if [ $? -ne 0 ]; then
            echo "エラー: ダウンロードプロセス (PID: $pid) が失敗しました。"
            exit 1
        fi
    done
    echo "--- 全てのソースコードのダウンロードが完了 ---"
    echo ""

    # --- 展開フェーズ (並列ダウンロードが有効な場合のみ) ---
    echo "--- 全てのソースコードの展開を開始 ---"
    for ((i=0; i<${#packages[@]}; i+=3)); do
        name="${packages[$i]}"
        url="${packages[$i+1]}"
        tarball_name=$(basename "$url")
        extracted_dir=$(echo "$tarball_name" | sed -E 's/(\.tar\.xz|\.tar\.gz|\.tgz|\.tar\.bz2)//')

        echo "展開中: $tarball_name"
        rm -rf "$DOWNLOAD_DIR/$extracted_dir"
        tar -xf "$DOWNLOAD_DIR/$tarball_name" -C "$DOWNLOAD_DIR" || { echo "エラー: $tarball_name の展開に失敗しました。"; exit 1; }

        extracted_dirs["$name"]="$extracted_dir"
    done
    echo "--- 全てのソースコードの展開が完了 ---"
    echo ""
fi

# --- パッケージビルド用のヘルパー関数 ---
# 引数: name, url, configure_options
build_package() {
    local name="$1"
    local url="$2"
    local config_options="$3"

    echo "--- $name のビルドを開始 ---"
    echo "ソースダウンロードURL: $url"

    local tarball_name=$(basename "$url")
    local extracted_dir=$(echo "$tarball_name" | sed -E 's/(\.tar\.xz|\.tar\.gz|\.tgz|\.tar\.bz2)//')

    # 並列ダウンロードが無効な場合、ここでダウンロードと展開を行う
    if ! $PARALLEL_DOWNLOAD; then
        # ダウンロード
        if [ ! -f "$DOWNLOAD_DIR/$tarball_name" ]; then
            echo "ダウンロード中: $tarball_name"
            wget -P "$DOWNLOAD_DIR" "$url" || { echo "エラー: $tarball_name のダウンロードに失敗しました。"; exit 1; }
        else
            echo "すでにダウンロード済み: $tarball_name"
        fi

        # 展開
        echo "展開中: $tarball_name"
        rm -rf "$DOWNLOAD_DIR/$extracted_dir"
        tar -xf "$DOWNLOAD_DIR/$tarball_name" -C "$DOWNLOAD_DIR" || { echo "エラー: $tarball_name の展開に失敗しました。"; exit 1; }
    fi

    echo "ソースディレクトリ: $DOWNLOAD_DIR/$extracted_dir"

    # ビルドディレクトリへ移動
    cd "$DOWNLOAD_DIR/$extracted_dir" || { echo "エラー: $DOWNLOAD_DIR/$extracted_dir ディレクトリへの移動に失敗しました。"; exit 1; }

    # config_options を評価して最終的なオプション文字列を生成
    local final_config_options=$(eval echo "$config_options")

    # 設定 (configure)
    echo "設定中 ($name): ./configure --prefix=$INSTALL_DIR/$name $final_config_options"
    ./configure --prefix="$INSTALL_DIR/$name" $final_config_options || { echo "エラー: $name の configure に失敗しました。"; exit 1; }

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
echo "--- 依存関係のビルドを開始 ---"

for ((i=0; i<${#packages[@]}; i+=3)); do
    name="${packages[$i]}"
    url="${packages[$i+1]}"
    config_options="${packages[$i+2]}"

    # `build_package` には常に name, url, config_options を渡す
    # 関数内部で parallel_download フラグをチェックして処理を分岐
    build_package "$name" "$url" "$config_options"
done

echo "============================================================"
echo "すべての依存関係のビルドとインストールが完了しました。"
echo "次にbinutils-gdbのビルドスクリプトを実行できます。"
echo "============================================================"