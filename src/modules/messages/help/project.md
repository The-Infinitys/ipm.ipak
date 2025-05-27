# {name} プロジェクトコマンド

`project` コマンドは、{name}を使用してプロジェクトの作成、ビルド、インストール、削除などを行うためのコマンドです。

## 使用方法

以下の形式でコマンドを実行します:

    {name} project <サブコマンド> [オプション]

## サブコマンド

- **create** または **new**: 新しいプロジェクトを作成します。

  - 使用例: `{name} project create --name my_project --template rust`
  - オプション:
    - `--name` または `--project-name`: プロジェクト名を指定します。
    - `--template`: プロジェクトテンプレートを指定します（例: `rust`, `python`）。
    - `--author-name`: 作成者の名前を指定します。
    - `--author-email`: 作成者のメールアドレスを指定します。

- **build** または **compile**: プロジェクトをビルドします。

  - 使用例: `{name} project build --release`
  - オプション:
    - `--release`: リリースモードでビルドします。
    - `--debug`: デバッグモードでビルドします。
    - `--shell`: 使用するシェルを指定します（例: `bash`, `zsh`）。

- **install**: プロジェクトをインストールします。

  - 使用例: `{name} project install --global`
  - オプション:
    - `--global`: グローバルモードでインストールします。
    - `--local`: ローカルモードでインストールします。
    - `--shell`: 使用するシェルを指定します。

- **remove**: プロジェクトを削除します。

  - 使用例: `{name} project remove --local`
  - オプション:
    - `--global`: グローバルモードで削除します。
    - `--local`: ローカルモードで削除します。
    - `--shell`: 使用するシェルを指定します。

- **purge**: プロジェクトを完全に削除します（設定ファイルなども含む）。

  - 使用例: `{name} project purge --global`
  - オプション:
    - `--global`: グローバルモードで削除します。
    - `--local`: ローカルモードで削除します。
    - `--shell`: 使用するシェルを指定します。

- **metadata** または **info**: プロジェクトのメタデータを表示します。

  - 使用例: `{name} project metadata`

- **package** または **pkg**: プロジェクトをパッケージ化します。
  - 使用例: `{name} project package --target x86_64`
  - オプション:
    - `--target`: パッケージのターゲットアーキテクチャを指定します。

## 注意事項

- サブコマンドによっては、管理者権限が必要な場合があります。
- プロジェクト名やテンプレート名は正確に指定してください。

## 詳細情報

{name}の詳細なドキュメントや最新情報については、公式ウェブサイトをご覧ください。

Believe in The Infinite Possibilities!  
The Infinity's
