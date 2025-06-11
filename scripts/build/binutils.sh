#!/bin/bash
# build_binutils_gdb_custom_deps.sh

# エラーが発生した場合、スクリプトを停止する
set -e

# --- 設定 ---
# 依存関係がインストールされているディレクトリ
DEPS_INSTALL_DIR=$(pwd)/lib/deps/opt_deps

# カスタムビルドされたflexとbisonがPATHに含まれていることを確認
# これにより、binutils-gdbのビルドプロセスで正しく参照されます
export PATH="$DEPS_INSTALL_DIR/flex/bin:$DEPS_INSTALL_DIR/bison/bin:$PATH"

echo "============================================================"
echo "カスタムビルドされた依存関係を使用してbinutils-gdbをビルドします..."
echo "============================================================"
echo ""

# binutils-gdbのソースディレクトリへ移動
# スクリプトの実行場所とソースディレクトリの相対パスを確認してください
cd ./lib/src/binutils-gdb/ || { echo "エラー: ./lib/src/binutils-gdb/ ディレクトリが見つかりません。パスを確認してください。"; exit 1; }

# 既存のビルドディレクトリをクリーンアップし、新しく作成
if [ -d "build" ]; then
  echo "既存のビルドディレクトリを削除中..."
  rm -rf build
fi
mkdir build
cd build || { echo "エラー: build ディレクトリへの移動に失敗しました。"; exit 1; }

# binutils-gdb 自体のインストールプレフィックスを定義
# ここでは、元のスクリプトの論理を維持しています
PREFIX=$(pwd)/../../../build/binutils-gdb
echo "binutils-gdbのインストール先: $PREFIX"
echo ""

# configure を実行し、カスタム依存関係のパスを指定
echo "configureを実行中..."
../configure --prefix="$PREFIX" \
             --disable-nls \
             --disable-werror \
             --enable-gdb \
             --with-gmp="$DEPS_INSTALL_DIR/gmp" \
             --with-mpfr="$DEPS_INSTALL_DIR/mpfr" \
             --with-zlib="$DEPS_INSTALL_DIR/zlib" \
             --with-isl="$DEPS_INSTALL_DIR/isl" \
             --with-libelf-dir="$DEPS_INSTALL_DIR/elfutils" \
             || { echo "エラー: binutils-gdb の configure に失敗しました。"; exit 1; }

echo ""
# コンパイル
echo "コンパイル中 (make -j$(nproc))..."
make -j$(nproc) || { echo "エラー: binutils-gdb の make に失敗しました。"; exit 1; }

echo ""
# インストール
echo "インストール中 (make install)..."
make install || { echo "エラー: binutils-gdb の make install に失敗しました。"; exit 1; }

echo "============================================================"
echo "binutils-gdbのビルドとインストールが完了しました。"
echo "インストールされたパス: $PREFIX"
echo "============================================================"
