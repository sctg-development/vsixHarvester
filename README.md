# vsixHarvester

## VSCode Extension Downloader in Rust

This Rust program reads the `recommendations` array from an `extensions.json` file and downloads the corresponding VSIX packages for Visual Studio Code extensions.

### Features

- Reads a list of extensions from `extensions.json`.
- Downloads the latest version of each extension as a VSIX package.
- Supports proxy configuration.
- Option to force re-download even if the file already exists.
- Provides verbose output for detailed logging.

### Prerequisites

- **Rust** and **Cargo** installed on your system. You can install them from [rustup.rs](https://rustup.rs/).

### Installation

```sh
cargo install vsixHarvester
```

### Usage

```sh
vsixHarvester [OPTIONS]
```

#### Options

- `-i`, `--input <INPUT>`: Path to the `extensions.json` file. Default is `./.vscode/extensions.json`.
- `-d`, `--destination <DESTINATION>`: Destination folder to save the VSIX files. Default is `./.vscode/extensions`.
- `--no-cache`: Force re-download even if the extension file already exists.
- `--proxy <PROXY>`: Proxy URL to use for HTTP requests.
- `-v`, `--verbose`: Enable verbose output for detailed logging.
- `-h`, `--help`: Print help information.
- `--os-arch <OS-ARCH>`: OS architecture to install the extensions for.

#### Example

```sh
vsixHarvester \
  --input ./your/path/to/extensions.json \
  --destination ./your/path/to/extensions \
  --no-cache \
  --os-arch win32-x64 \
  --verbose
```

##### OS architecture options

- `win32-x64`
- `win32-arm64`
- `darwin-x64`
- `darwin-arm64`
- `linux-x64`
- `linux-arm64`

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
