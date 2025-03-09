# vsixHarvester

## VSCode Extension Downloader in Rust

This Rust program reads the different arrays from an `extensions.json` file and downloads the corresponding VSIX packages for Visual Studio Code extensions.  
This is an adaptation of the original [vsixHarvester](https://github.com/ShortArrow/vsixHarvester), it does not use the Visual Studio Code `extensions.json` file, but a custom one. The custom `extensions.json` file allows to specify the architecture for each extension.

### Features

- Reads a list of extensions from `extensions.json`.
- Downloads the latest version of each extension as a VSIX package.
- Supports proxy configuration.
- Option to force re-download even if the file already exists.
- Provides verbose output for detailed logging.
- Direct download of a single extension without using extensions.json file.
- Parrallel download of extensions.

### Binaries

- [Windows](https://github.com/sctg-development/vsixHarvester/releases/download/0.2.5/vsixHarvester_windows_amd64_0.2.5.exe)
- [macOS AMD64](https://github.com/sctg-development/vsixHarvester/releases/download/0.2.5/vsixHarvester_macos_amd64_0.2.5)
- [macOS ARM64](https://github.com/sctg-development/vsixHarvester/releases/download/0.2.5/vsixHarvester_macos_arm64_0.2.5)
- [Linux static AMD64](https://github.com/sctg-development/vsixHarvester/releases/download/0.2.5/vsixHarvester_linux_amd64_static_0.2.5)
- [Linux static ARM64](https://github.com/sctg-development/vsixHarvester/releases/download/0.2.5/vsixHarvester_linux_arm64_static_0.2.5)
- [Linux static ARM32](https://github.com/sctg-development/vsixHarvester/releases/download/0.2.5/vsixHarvester_linux_armhf_static_0.2.5)
  
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

- `-i`, `--input <INPUT>`: Path to the `extensions.json` file. Default is `./extensions.json`.
- `-d`, `--destination <DESTINATION>`: Destination folder to save the VSIX files. Default is `./extensions`.
- `-D`, `--download <EXTENSION>`: Download a single extension (e.g., 'golang.Go') without using extensions.json.
- `-a`, `--arch <ARCHITECTURE>`: Architecture for single extension download (e.g., 'linux_x64', 'darwin_arm64').
- `--no-cache`: Force re-download even if the extension file already exists.
- `--proxy <PROXY>`: Proxy URL to use for HTTP requests.
- `--serial-download`: Download extensions serially instead of in parallel.
- `-v`, `--verbose`: Enable verbose output for detailed logging.
- `-h`, `--help`: Print help information.

#### Environment Variables

Alternatively, you can set the following environment variables:

- EXTENSIONS_FILE (default: `./extensions.json`)
- OUTPUT_DIR (default: `./extensions`)
- PROXY (default: none)
- VERBOSE (default: false) - sets the log level to `info`
- DOWNLOAD (default: none)
- ARCH (default: none)
- SERIAL_DOWNLOAD (default: false)
- NO_CACHE (default: false)
  
#### Logging

The program use `env_logger` for logging. You can set the `RUST_LOG` environment variable to control the log level.

#### Example

```sh
vsixHarvester \
  --input ./your/path/to/extensions.json \
  --destination ./your/path/to/extensions \
  --no-cache \
  --verbose
```

Direct download of a single extension:

```sh
vsixHarvester --download golang.Go --destination ./extensions
```

Direct download with specific architecture:

```sh
vsixHarvester -D ms-python.python -a linux_x64 -d ./extensions
```

##### Architecture options

- `win32_x64`
- `win32_arm64`
- `darwin_x64`
- `darwin_arm64`
- `linux_x64`
- `linux_arm64`

### extensions.json Format

The `extensions.json` file should have the following structure:

```json
{
  "universal": [
    // "publisher.extensionName",
    // "anotherPublisher.anotherExtensionName",
    "GitHub.copilot",
    "GitHub.copilot-chat",
    "golang.Go",
    "yzhang.markdown-all-in-one",
    "eamodio.gitlens"
  ],
  "linux_x64":[
    "rust-lang.rust-analyzer",
    "ms-python.python"
  ],
  "linux_arm64":[
    "rust-lang.rust-analyzer",
    "ms-python.python"
  ]
}
```

### Thanks

- Inspired from [offvsix](https://github.com/exaluc/offvsix)
- Modified from [vsixHarvester](https://github.com/ShortArrow/vsixHarvester)
