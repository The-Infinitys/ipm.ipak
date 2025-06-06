# {name} manual
## コマンド
### 基本コマンド
- **help**  
  このヘルプメッセージを表示します。  
  使用例: `{name} help`

- **version**  
  {name} のバージョン情報を表示します。  
  使用例: `{name} version`
- **manual**
  {name} のマニュアルを表示します。
  使用例: `{name} manual`

### project - プロジェクト管理
- **create**  
  新しいプロジェクトを作成します。  
  使用例: `{name} project create --project-name <名前> [--template <default|rust>]`

- **build**  
  プロジェクトをビルドします。  
  使用例: `{name} project build [--release] [--shell <bash|zsh>]`

- **install**  
  プロジェクトをインストールします。  
  使用例: `{name} project install [--global]`

- **remove**  
  プロジェクトを削除します。  
  使用例: `{name} project remove`

- **purge**  
  プロジェクトと関連データを完全に削除します。  
  使用例: `{name} project purge`

- **package**  
  プロジェクトをパッケージ化します。  
  使用例: `{name} project package`

- **metadata**  
  プロジェクトのメタデータを表示します。  
  使用例: `{name} project metadata`
- **run**
  プロジェクトで任意のコマンドを実行します。  
  使用例: `{name} project run [command]`

### system - システム設定
- **configure**  
  ローカルまたはグローバル設定を構成します。  
  使用例: `{name} system configure [--local|--global]`

### package (pkg) - パッケージ管理
- **list**  
  インストール済みパッケージを表示します。  
  使用例: `{name} pkg list [--local|--global]`
- **install**  
  パッケージをインストールします。
  使用例: `{name} pkg install [file-path] [--local|--global]`
- **remove**  
  パッケージを削除します。バイナリのみが削除され、設定ファイルは残ったままになります。
  使用例: `{name} pkg remove [package-name] [--local|--global]`
- **purge**
  パッケージを削除します。設定ファイルも含めて完全に削除されます。
  使用例: `{name} pkg purge [package-name] [--local|--global]`

## 詳細情報
{name} は、プロジェクトの作成、ビルド、インストールを簡単に行えるツールです。  
ローカル（~/.ipak）またはグローバル（/etc/ipak）でのパッケージ管理をサポートし、依存関係の解決やバージョン管理も可能です。  
さらに詳しい情報は、公式ドキュメントを参照してください。
