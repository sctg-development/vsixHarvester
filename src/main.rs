mod architecture;
mod args;
mod config;
mod error;
mod extension;
mod marketplace;
mod types;
#[cfg(test)]
mod tests;

use architecture::Architecture;
use args::{Args, Parser};
use config::MAX_CONCURRENT_DOWNLOADS;

use error::{Result, VsixHarvesterError};
use futures::stream::{self, StreamExt};
use marketplace::download_extension;

use env_logger;
use log::{error, info};
use std::fs;
use std::path::Path;
use tokio;

use extension::{Extension, Extensions};

/// Create a directory if it does not exist
///
/// # Arguments
///
/// * `path` - The path of the directory to create
///
/// # Returns
///
/// A Result indicating success or an error that occurred
pub(crate) fn create_directory_if_not_exists(path: &str) -> Result<()> {
    let path = Path::new(path);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Process extensions based on the provided arguments
///
/// # Arguments
///
/// * `args` - The command line arguments
///
/// # Returns
///
/// A Result indicating success or an error that occurred
pub(crate) async fn process_extensions(args: &Args) -> Result<()> {
    //let args = Args::parse();

    // Handle direct extension download if specified
    if let Some(str_extension) = &args.download {
        let extension = Extension::from_id(str_extension)?;
        return download_single_extension(extension, args).await;
    } else {
        return download_extensions_from_json(args).await;
    }
}

/// Download a single extension
///
/// # Arguments
/// * `extension` - The extension to download
/// * `args` - The command line arguments
///
/// # Returns
///
/// A Result indicating success or an error that occurred
async fn download_single_extension(extension: Extension<'_>, args: &Args) -> Result<()> {
    info!("Direct download mode for extension: {}", extension.to_id());
    // Map architecture to target platform
    let target_platform = args
        .arch
        .as_deref()
        .and_then(Architecture::from_cli_arg)
        .and_then(|arch| arch.to_target_platform());

    if target_platform.is_some() {
        info!("Using architecture: {}", target_platform.unwrap());
    } else {
        info!("Using universal architecture");
    }

    // Ensure the destination directory exists
    create_directory_if_not_exists(&args.destination)?;

    // Download the extension
    if let Err(e) = download_extension(
        extension.clone(),
        &args.destination,
        args.no_cache,
        args.proxy.as_deref(),
        target_platform,
        args.engine_version.as_deref()
    )
    .await
    {
        error!(
            "Error occurred when downloading {}: {}",
            extension.to_id(),
            e
        );
        return Err(e);
    }

    return Ok(());
}

/// Download extensions from extensions.json
///
/// # Arguments
///
/// * `args` - The command line arguments
///
/// # Returns
///
/// A Result indicating success or an error that occurred
async fn download_extensions_from_json(args: &Args) -> Result<()> {
    // Read extensions.json
    info!("Attempting to read file: {}", &args.input);
    let file_content = match fs::read_to_string(&args.input) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read file {}: {}", &args.input, e);
            return Err(VsixHarvesterError::IoError(e));
        }
    };
    let extensions: Extensions = match serde_json::from_str(&file_content) {
        Ok(extensions) => extensions,
        Err(e) => {
            error!("Failed to parse file {}: {}", &args.input, e);
            return Err(VsixHarvesterError::JsonError(e));
        }
    };

    // Ensure the destination directory exists
    create_directory_if_not_exists(&args.destination)?;

    // Define all platform categories with their target platform identifiers
    let platforms = Architecture::available_architectures();

    // Process extensions for each platform
    for (platform_field, target_platform) in platforms {
        // Use reflection to get the field from the extensions struct
        let extensions_list = Architecture::get_extensions_list(platform_field, &extensions);

        // Process the extensions for this platform if any
        if let Some(platform_extensions) = extensions_list {
            let mut tasks = Vec::new();
            for str_extension in platform_extensions {
                let extension = Extension::from_id(str_extension)?;
                info!("Attempting to download extension: {}", extension.to_id());
                let task = download_extension(
                    extension.clone(),
                    &args.destination,
                    args.no_cache,
                    args.proxy.as_deref(),
                    target_platform,
                    args.engine_version.as_deref()
                );
                tasks.push(task);
            }
            let concurrent_downloads = if args.serial {
                1
            } else {
                MAX_CONCURRENT_DOWNLOADS
            };
            let mut stream = stream::iter(tasks).buffer_unordered(concurrent_downloads);
            while let Some(result) = stream.next().await {
                if let Err(e) = result {
                    error!("Error occurred when downloading: {}", e);
                }
            }
        }
    }
    return Ok(());
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if args.verbose {
        // set log level to info if verbose flag is set and RUST_LOG is not already set
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();
    process_extensions(&args).await
}
