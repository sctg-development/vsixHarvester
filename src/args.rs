pub use clap::Parser;
use crate::config::{DEFAULT_FILE_NAME, DEFAULT_PATH, VERSION};



#[derive(Parser)]
#[command(
    version = VERSION,
    about = "Download VSCode extensions for offline use"
)]
pub struct Args {
    /// Path to extensions.json
    #[arg(short, long, default_value_t = format!("./{}", DEFAULT_FILE_NAME), env = "EXTENSIONS_FILE")]
    pub input: String,

    /// Output directory
    #[arg(short, long, default_value_t = format!("./{}", DEFAULT_PATH), env = "OUTPUT_DIR")]
    pub destination: String,

    /// Force redownload if exists
    #[arg(long, default_value = "false", env = "NO_CACHE")]
    pub no_cache: bool,

    /// Specify proxy url
    #[arg(long, env = "PROXY")]
    pub proxy: Option<String>,

    /// Show verbose infomation
    #[arg(short, long, default_value = "false", env = "VERBOSE")]
    pub verbose: bool,

    /// Download a single extension (e.g., 'golang.Go')
    #[arg(short = 'D', long = "download", env = "DOWNLOAD")]
    pub download: Option<String>,

    /// Architecture for single extension download (e.g., 'linux_x64', 'darwin_arm64')
    #[arg(short, long, env = "ARCH")]
    pub arch: Option<String>,

    /// Engine version to be compatible with
    #[arg(short, long, env)]
    pub engine_version: Option<String>,

    /// Disable parallel downloads
    #[arg(
        long = "serial-download",
        default_value = "false",
        env = "SERIAL_DOWNLOAD"
    )]
    pub serial: bool,
}