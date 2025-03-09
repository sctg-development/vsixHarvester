use clap::Parser;
use serde::Deserialize;
use serde_json::json;
use futures::stream::{self, StreamExt};
use std::fs;
use std::path::Path;
use tokio;
use thiserror::Error;

const API_URL: &str = "https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery";
const MARKETPLACE_URL: &str = "https://marketplace.visualstudio.com/_apis/public/gallery/publishers";
const VERSION: &str = "0.2.3";
const USER_AGENT: &str = "Offline VSIX/0.2.3";
const MARKETPLACE_API_VERSION: &str = "3.0-preview.1";
const DEFAULT_FILE_NAME: &str = "extensions.json";
const DEFAULT_PATH: &str = "./extensions";
const MAX_CONCURRENT_DOWNLOADS: usize = 5;

#[derive(Error, Debug)]
pub enum VsixHarvesterError {
    #[error("Invalid extension identifier: {0}")]
    InvalidExtensionId(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Failed to query marketplace API: {0}")]
    ApiError(String),
    
    #[error("Failed to download extension: {0}")]
    DownloadError(String),
}
pub type Result<T> = std::result::Result<T, VsixHarvesterError>;

#[derive(Clone)]
struct Extension<'a> {
    publisher: &'a str,
    name: &'a str,
}

impl<'a> Extension<'a> {
    fn from_id(id: &'a str) -> std::result::Result<Self, VsixHarvesterError> {
        let parts: Vec<&str> = id.split('.').collect();
        if parts.len() != 2 {
            return Err(VsixHarvesterError::InvalidExtensionId(id.to_string()));
        }
        Ok(Self {
            publisher: parts[0],
            name: parts[1],
        })
    }
    
    fn to_id(&self) -> String {
        format!("{}.{}", self.publisher, self.name)
    }

    #[allow(dead_code)]
    fn to_string(&self) -> String {
        format!("Publisher: {}, Name: {}", self.publisher, self.name)
    }
}

#[derive(Parser)]
#[command(
    version = VERSION,
    about = "Download VSCode extensions for offline use"
)]
struct Args {
    /// Path to extensions.json
    #[arg(short, long, default_value_t = format!("./{}", DEFAULT_FILE_NAME))]
    input: String,

    /// Output directory
    #[arg(short, long, default_value_t = format!("./{}", DEFAULT_PATH))]
    destination: String,

    /// Force redownload if exists
    #[arg(long)]
    no_cache: bool,

    /// Specify proxy url
    #[arg(long)]
    proxy: Option<String>,

    /// Show verbose infomation
    #[arg(short, long)]
    verbose: bool,

    /// Download a single extension (e.g., 'golang.Go')
    #[arg(short = 'D', long = "download")]
    download: Option<String>,

    /// Architecture for single extension download (e.g., 'linux_x64', 'darwin_arm64')
    #[arg(short, long)]
    arch: Option<String>,

    /// Disable parallel downloads
    #[arg(long = "serial-download", default_value = "false")]
    serial: bool,
}

#[derive(Deserialize)]
struct Extensions {
    universal: Option<Vec<String>>,
    linux_x64: Option<Vec<String>>,
    linux_arm64: Option<Vec<String>>,
    darwin_x64: Option<Vec<String>>,
    darwin_arm64: Option<Vec<String>>,
    win32_x64: Option<Vec<String>>,
    win32_arm64: Option<Vec<String>>,
}

/// Create a directory if it does not exist
///
/// # Arguments
///
/// * `path` - The path of the directory to create
///
/// # Returns
///
/// A Result indicating success or an error that occurred
fn create_directory_if_not_exists(path: &str) -> Result<()> {
    let path = Path::new(path);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Downloads a VSCode extension by its identifier
/// 
/// # Arguments
/// 
/// * `extension` - The extension to downloads
/// * `destination` - The directory where the extension will be saved
/// * `no_cache` - Whether to force redownload even if the extension already exists
/// * `proxy` - Optional proxy URL
/// * `verbose` - Whether to print verbose output
/// * `os_arch` - Optional target platform
/// 
/// # Returns
/// 
/// A Result indicating success or an error that occurred
async fn download_extension(
    extension: Extension<'_>,
    destination: &str,
    no_cache: bool,
    proxy: Option<&str>,
    verbose: bool,
    os_arch: Option<&str>,
) -> Result<()> {
    if verbose {
        println!("Progress in extension: {}", extension.to_id());
    }

    // Get latest version
    let version = get_extension_version(extension.clone(), proxy, verbose).await?;
    if verbose {
        println!("Latest version of {}: {}", extension.to_id(), version);
    }

    let (download_url, file_path) =
    build_download_url_and_file_path(extension.clone(), &version, destination, os_arch);


    if verbose {
        println!("Download URL: {}", download_url);
    }

    // Make file path

    // Check file already exists
    if !no_cache && Path::new(&file_path).exists() {
        if verbose {
            println!(
                "Skip download: File is already exists. File Name {}.",
                file_path
            );
        }
        return Ok(());
    }

    // Create http client
    let client_builder = reqwest::Client::builder();
    let client = if let Some(proxy_url) = proxy {
        if verbose {
            println!("Using proxy: {}", proxy_url);
        }
        let proxy = reqwest::Proxy::all(proxy_url)?;
        client_builder.gzip(true).proxy(proxy).build()?
    } else {
        client_builder.gzip(true).build()?
    };

    // Download VSIX file
    if verbose {
        println!("Download form {}", download_url);
    }
    let resp = client
        .get(&download_url)
        .header(reqwest::header::ACCEPT_ENCODING, "gzip")
        .send()
        .await?;
    if !resp.status().is_success() {
        eprintln!("Fail download of {}", extension.to_id());
        return Err(VsixHarvesterError::DownloadError(extension.to_id()));
    }

    let vsix_raw_content = resp.bytes().await?;

    // Save file
    fs::write(&file_path, &vsix_raw_content)?;
    if verbose {
        println!("Saved in {}", file_path);
    }

    Ok(())
}

/// Get the latest version of a VSCode extension
///
/// # Arguments
///
/// * `extension` - The extension to get the version of
/// * `proxy` - Optional proxy URL
/// * `verbose` - Whether to print verbose output
///
/// # Returns
///
/// A Result containing the version or an error that occurreds
async fn get_extension_version(
    extension: Extension<'_>,
    proxy: Option<&str>,
    verbose: bool,
) -> std::result::Result<String, VsixHarvesterError> {
    let api_url = API_URL;
    let payload = json!({
        "filters": [{
            "criteria": [
                {"filterType": 7, "value": format!("{}.{}", extension.publisher, extension.name)}
            ]
        }],
        "flags": 914
    });

    // Create http client
    let client_builder = reqwest::Client::builder();
    let client = if let Some(proxy_url) = proxy {
        if verbose {
            println!("Using proxy for API request: {}", proxy_url);
        }
        let proxy = reqwest::Proxy::all(proxy_url)?;
        client_builder.proxy(proxy).build()?
    } else {
        client_builder.build()?
    };

    // Send POST request
    if verbose {
        println!(
            "Sending query for Marketplace API: {}.{}",
            extension.publisher, extension.name
        );
    }
    let resp = client
        .post(api_url)
        .header("Content-Type", "application/json")
        .header("Accept", format!("application/json;api-version={}", MARKETPLACE_API_VERSION))
        .header("User-Agent", USER_AGENT)
        .json(&payload)
        .send()
        .await?;

    if !resp.status().is_success() {
        eprintln!("Failed query for Marketplace API");
        return Err(VsixHarvesterError::ApiError("Failed query for Marketplace API".to_string()));
    }

    let resp_json: serde_json::Value = resp.json().await?;

    // Extract version
    let version = resp_json["results"][0]["extensions"][0]["versions"][0]["version"]
        .as_str()
        .ok_or_else(|| VsixHarvesterError::ApiError("Failed to get extension version".to_string()))?
        .to_string();

    Ok(version)
}

/// Build the download URL and file path for a VSCode extension
///
/// # Arguments
///
/// * `extension` - The extension to build the URL and file path for
/// * `version` - The version of the extension
/// * `destination` - The directory where the extension will be saved
/// * `os_arch` - Optional target platform
///
/// # Returns
///
/// A tuple containing the download URL and file path
fn build_download_url_and_file_path(
    extension: Extension<'_>,
    version: &str,
    destination: &str,
    os_arch: Option<&str>,
) -> (String, String) {
    let file_name: String;
    let file_path: String;
    let download_url: String;

    if let Some(target_platform) = os_arch {
        file_name = format!("{}.{}-{version}@{}.vsix", extension.publisher, extension.name, target_platform);
        file_path = format!("{}/{}", destination, file_name);
        download_url = format!(
            "{}/{}/vsextensions/{}/{}/vspackage?targetPlatform={}",
            MARKETPLACE_URL, extension.publisher, extension.name, version, target_platform
        );
    } else {
        file_name = format!("{}.{}-{}.vsix", extension.publisher, extension.name, version);
        file_path = format!("{}/{}", destination, file_name);
        download_url = format!(
            "{}/{}/vsextensions/{}/{}/vspackage",
            MARKETPLACE_URL, extension.publisher, extension.name, version
        );
    }

    (download_url, file_path)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle direct extension download if specified
    if let Some(str_extension) = &args.download {
        let extension = Extension::from_id(str_extension)?;
        if args.verbose {
            println!("Direct download mode for extension: {}", extension.to_id());
        }
        // Map architecture to target platform
        let target_platform = match args.arch.as_deref() {
            Some("linux_x64") => Some("linux-x64"),
            Some("linux_arm64") => Some("linux-arm64"),
            Some("darwin_x64") => Some("darwin-x64"),
            Some("darwin_arm64") => Some("darwin-arm64"),
            Some("win32_x64") => Some("win32-x64"),
            Some("win32_arm64") => Some("win32-arm64"),
            Some(arch) => {
                eprintln!("Unknown architecture: {}. Using universal instead.", arch);
                None
            }
            None => None, // Universal/default
        };

        if args.verbose && target_platform.is_some() {
            println!("Using architecture: {}", target_platform.unwrap());
        } else if args.verbose {
            println!("Using universal architecture");
        }

        // Ensure the destination directory exists
        create_directory_if_not_exists(&args.destination)?;

        // Download the extension
        if let Err(e) = download_extension(
            extension,
            &args.destination,
            args.no_cache,
            args.proxy.as_deref(),
            args.verbose,
            target_platform,
        )
        .await
        {
            eprintln!("Error occurred when downloading {}: {}", str_extension, e);
            return Err(e);
        }

        return Ok(());
    }

    // Read extensions.json
    if args.verbose {
        println!("Attempting to read file: {}", &args.input);
    }
    let file_content = match fs::read_to_string(&args.input) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file {}: {}", &args.input, e);
            return Err(VsixHarvesterError::IoError(e));
        }
    };
    let extensions: Extensions = match serde_json::from_str(&file_content) {
        Ok(extensions) => extensions,
        Err(e) => {
            eprintln!("Failed to parse file {}: {}", &args.input, e);
            return Err(VsixHarvesterError::JsonError(e));
        }
    };

    // Ensure the destination directory exists
    create_directory_if_not_exists(&args.destination)?;

    // Define all platform categories with their target platform identifiers
    let platforms = [
        ("universal", None),
        ("linux_x64", Some("linux-x64")),
        ("linux_arm64", Some("linux-arm64")),
        ("darwin_x64", Some("darwin-x64")),
        ("darwin_arm64", Some("darwin-arm64")),
        ("win32_x64", Some("win32-x64")),
        ("win32_arm64", Some("win32-arm64")),
    ];

    // Process extensions for each platform
    for (platform_field, target_platform) in platforms {
        // Use reflection to get the field from the extensions struct
        let extensions_list = match platform_field {
            "universal" => &extensions.universal,
            "linux_x64" => &extensions.linux_x64,
            "linux_arm64" => &extensions.linux_arm64,
            "darwin_x64" => &extensions.darwin_x64,
            "darwin_arm64" => &extensions.darwin_arm64,
            "win32_x64" => &extensions.win32_x64,
            "win32_arm64" => &extensions.win32_arm64,
            _ => continue, // Skip unknown platforms
        };

        // Process the extensions for this platform if any
        if let Some(ext_list) = extensions_list {
            let mut tasks = Vec::new();
            for str_extension in ext_list {
                let extension = Extension::from_id(str_extension)?;
                if args.verbose {
                    println!("Attempting to download extension: {}", extension.to_id());
                }
                let task = download_extension(
                    extension.clone(),
                    &args.destination,
                    args.no_cache,
                    args.proxy.as_deref(),
                    args.verbose,
                    target_platform,
                );
                tasks.push(task);
            }
            let concurrent_downloads = if args.serial { 1 } else { MAX_CONCURRENT_DOWNLOADS };
            let mut stream = stream::iter(tasks).buffer_unordered(concurrent_downloads);
            while let Some(result) = stream.next().await {
                if let Err(e) = result {
                    eprintln!("Error occurred when downloading: {}", e);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_from_id() {
        let ext = Extension::from_id("publisher.name").unwrap();
        assert_eq!(ext.publisher, "publisher");
        assert_eq!(ext.name, "name");
    }
    
    #[test]
    fn test_invalid_extension_id() {
        let result = Extension::from_id("invalid");
        assert!(result.is_err());
    }
}