# About Infinite Package Project

## Scripts

`ipak/scripts` ディレクトリに入っているスクリプトファイルは、
パッケージを操作する際に使用するものです。

### 実行

すべてのシェルスクリプトは、プロジェクトのルートディレクトリで行われます。
つまり、project.yamlが存在するフォルダがカレントディレクトリになるということです。
 /
| - ipak/
|      | - scripts/
|      |         | - install.sh
|      |         | - remove.sh
|      |         | - build.sh
|      |         | - purge.sh
|      | - project-ignore.yaml
| - project.yaml
| - src/
      | ...


### 環境変数

#### 共通
 - $IPAK_PROJECT_NAME :
    Infinite Packageのパッケージ名が代入されています。

 - $IPAK_PROJECT_VERSION :
    Infinite Packageのバージョンが代入されています。

#### 特有
 - $IPAK_BUILD_MODE :
    Infinite Packageをビルドする際のモードが代入されています。
        値: release / debug

 - $IPAK_INSTALL_MODE | $IPAK_REMOVE_MODE | $IPAK_PURGE_MODE :
    Infinite Packageを管理(インストール・削除・パージ)する際のモードが代入されています。
        値: local / global

### パッケージング

`ipak project package`
を実行することによって、プロジェクトをパッケージにすることができます。

```yaml
source-build:
  - log/
normal:
  - .gitignore
  - .git
min:
  - src/
  - target/debug/
  - target/release/*/
  - target/release/*.d
  - target/release/*rlib
  - Cargo*
```

project-ignore.yamlに無視するファイルやフォルダのデータを入れることができます。
