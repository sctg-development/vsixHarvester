# README.ja.md

## Rust製のVSCode拡張機能ダウンローダー

このRustプログラムは、`extensions.json`ファイルから`recommendations`配列を読み取り、対応するVisual Studio Code拡張機能のVSIXパッケージをダウンロードします。提供されたPythonスクリプトと同様の機能を持ちますが、Rustで実装されており、パフォーマンスと効率性が向上しています。

### 特徴

- `extensions.json`から拡張機能のリストを読み込む。
- 各拡張機能の最新バージョンをVSIXパッケージとしてダウンロード。
- プロキシ設定をサポート。
- ファイルが既に存在していても再ダウンロード可能。
- 詳細なログを表示するオプション。

### 前提条件

- システムに**Rust**と**Cargo**がインストールされていること。[rustup.rs](https://rustup.rs/)からインストールできます。

### 依存関係

以下のRustクレートを使用します：

- [`serde`](https://crates.io/crates/serde)
- [`serde_json`](https://crates.io/crates/serde_json)
- [`reqwest`](https://crates.io/crates/reqwest)
- [`tokio`](https://crates.io/crates/tokio)
- [`clap`](https://crates.io/crates/clap)

`Cargo.toml`に以下を指定してください：

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json", "cookies", "rustls-tls"] }
tokio = { version = "1", features = ["full"] }
clap = { version = "4.1", features = ["derive"] }
```

### インストール

1. **リポジトリをクローンするか、ソースコードをコピー**して新しいディレクトリに配置します。

2. **新しいCargoプロジェクトを初期化**（まだの場合）：

   ```sh
   cargo init vscode-extension-downloader
   ```

3. **`Cargo.toml`の依存関係を**上記のものに置き換えます。

4. **Rustのソースコードを** `src/main.rs`に配置します。

5. **プロジェクトをビルド**：

   ```sh
   cargo build --release
   ```

### 使用方法

```sh
./target/release/vscode-extension-downloader [OPTIONS]
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
./target/release/vscode-extension-downloader \
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

### ライセンス

このプロジェクトはMITライセンスの下で提供されています。

