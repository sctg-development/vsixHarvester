# README.md

## VSCode Extension Downloader in Rust

This Rust program reads the `recommendations` array from an `extensions.json` file and downloads the corresponding VSIX packages for Visual Studio Code extensions. It mimics the functionality of the provided Python script but is implemented in Rust for performance and efficiency.

### Features

- Reads a list of extensions from `extensions.json`.
- Downloads the latest version of each extension as a VSIX package.
- Supports proxy configuration.
- Option to force re-download even if the file already exists.
- Provides verbose output for detailed logging.

### Prerequisites

- **Rust** and **Cargo** installed on your system. You can install them from [rustup.rs](https://rustup.rs/).

### Dependencies

The program uses the following Rust crates:

- [`serde`](https://crates.io/crates/serde)
- [`serde_json`](https://crates.io/crates/serde_json)
- [`reqwest`](https://crates.io/crates/reqwest)
- [`tokio`](https://crates.io/crates/tokio)
- [`clap`](https://crates.io/crates/clap)

Ensure these are specified in your `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json", "cookies", "rustls-tls"] }
tokio = { version = "1", features = ["full"] }
clap = { version = "4.1", features = ["derive"] }
```

### Installation

1. **Clone the repository or copy the source code** into a new directory.

2. **Initialize a new Cargo project** if you haven't already:

   ```sh
   cargo init vscode-extension-downloader
   ```

3. **Replace the `Cargo.toml` dependencies** with the ones provided above.

4. **Place the Rust source code** into `src/main.rs`.

5. **Build the project**:

   ```sh
   cargo build --release
   ```

### Usage

```sh
./target/release/vscode-extension-downloader [OPTIONS]
```

#### Options

- `-i`, `--input <INPUT>`: Path to the `extensions.json` file. Default is `extensions.json`.
- `-d`, `--destination <DESTINATION>`: Destination folder to save the VSIX files. Default is `extensions`.
- `--no-cache`: Force re-download even if the extension file already exists.
- `--proxy <PROXY>`: Proxy URL to use for HTTP requests.
- `-v`, `--verbose`: Enable verbose output for detailed logging.
- `-h`, `--help`: Print help information.

#### Example

```sh
./target/release/vscode-extension-downloader \
  --input extensions.json \
  --destination extensions \
  --no-cache \
  --verbose
```

### extensions.json Format

The `extensions.json` file should have the following structure:

```json
{
  "recommendations": [
    "publisher.extensionName",
    "anotherPublisher.anotherExtensionName",
    // Add more extensions as needed
  ]
}
```

### Thanks

- Inspired from [offvsix](https://github.com/exaluc/offvsix)

### License

This project is licensed under the MIT License.
