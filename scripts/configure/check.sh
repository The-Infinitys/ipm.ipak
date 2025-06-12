#!/bin/bash
# check_download_urls.sh

# エラーが発生した場合、スクリプトを停止する
set -e

echo "==============================================="
echo "ダウンロードURLの存在を確認します..."
echo "==============================================="
echo ""

# --- パッケージ情報定義 (install_deps_from_source.sh と同じもの) ---
declare -a packages=(
    "gmp" "https://ftp.gnu.org/gnu/gmp/gmp-6.3.0.tar.xz"
    "mpfr" "https://www.mpfr.org/mpfr-current/mpfr-4.2.2.tar.gz"
    "zlib" "https://zlib.net/fossils/zlib-1.3.1.tar.gz"
    "isl" "https://libisl.sourceforge.io/isl-0.26.tar.xz"
    "elfutils" "https://sourceware.org/elfutils/ftp/0.190/elfutils-0.190.tar.bz2"
    "texinfo" "https://ftp.gnu.org/gnu/texinfo/texinfo-7.2.tar.gz"
    "libarchive" "https://www.libarchive.org/downloads/libarchive-3.7.4.tar.gz"
    "flex" "https://github.com/westes/flex/releases/download/v2.6.4/flex-2.6.4.tar.gz"
    "bison" "https://ftp.gnu.org/gnu/bison/bison-3.8.2.tar.xz"
)

ALL_URLS_OK=true

for ((i=0; i<${#packages[@]}; i+=2)); do # パッケージ名とURLの2つをセットで取得
    name="${packages[$i]}"
    url="${packages[$i+1]}"

    echo "確認中: $name から $url"
    if wget --spider --quiet "$url"; then
        echo "  [OK] 存在します。"
    else
        echo "  [エラー] 存在しないか、アクセスできません。"
        ALL_URLS_OK=false
    fi
done

echo ""
echo "==============================================="
if $ALL_URLS_OK; then
    echo "全てのURLが存在することを確認しました。メインスクリプトを実行できます。"
else
    echo "一部のURLが存在しないか、アクセスできません。URLを確認してください。"
    exit 1 # URL確認失敗としてスクリプトを終了
fi
echo "==============================================="