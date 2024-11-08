# vsixHarvester

## Rust製のVSCode拡張機能ダウンローダー

このRustプログラムは、`extensions.json`ファイルから`recommendations`配列を読み取り、対応するVisual Studio Code拡張機能のVSIXパッケージをダウンロードします。

### 特徴

- `extensions.json`から拡張機能のリストを読み込む。
- 各拡張機能の最新バージョンをVSIXパッケージとしてダウンロード。
- プロキシ設定をサポート。
- ファイルが既に存在していても再ダウンロード可能。
- 詳細なログを表示するオプション。

### 前提条件

- システムに**Rust**と**Cargo**がインストールされていること。[rustup.rs](https://rustup.rs/)からインストールできます。

### インストール

```sh
cargo install extHarvest
```

### 使用方法

```sh
extHarvest [OPTIONS]
```

#### オプション

- `-i`, `--input <INPUT>`：`extensions.json`ファイルへのパス。デフォルトは`extensions.json`。
- `-d`, `--destination <DESTINATION>`：VSIXファイルを保存するディレクトリ。デフォルトは`extensions`。
- `--no-cache`：拡張機能ファイルが既に存在していても再ダウンロードします。
- `--proxy <PROXY>`：HTTPリクエストに使用するプロキシURL。
- `-v`, `--verbose`：詳細なログを表示します。
- `-h`, `--help`：ヘルプ情報を表示。

#### 使用例

```sh
extHarvest \
  --input extensions.json \
  --destination extensions \
  --no-cache \
  --verbose
```

### extensions.jsonの形式

`extensions.json`ファイルは以下の構造である必要があります：

```json
{
  "recommendations": [
    "publisher.extensionName",
    "anotherPublisher.anotherExtensionName",
    // 必要に応じて拡張機能を追加
  ]
}
```

### 謝辞

- [offvsix](https://github.com/exaluc/offvsix) に影響を受けました。

