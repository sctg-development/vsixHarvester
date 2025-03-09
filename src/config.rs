pub const API_URL: &str =
    "https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery";
pub const MARKETPLACE_URL: &str =
    "https://marketplace.visualstudio.com/_apis/public/gallery/publishers";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const USER_AGENT: &str = concat!("Offline VSIX/", env!("CARGO_PKG_VERSION"));
pub const MARKETPLACE_API_VERSION: &str = "3.0-preview.1";
pub const DEFAULT_FILE_NAME: &str = "extensions.json";
pub const DEFAULT_PATH: &str = "./extensions";
pub const MAX_CONCURRENT_DOWNLOADS: usize = 5;
