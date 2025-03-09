mod architecture;
mod args;
mod config;
mod error;
mod extension;
mod marketplace;
#[cfg(test)]
mod tests;

use architecture::Architecture;
use args::{Args, Parser};
use config::MAX_CONCURRENT_DOWNLOADS;

use error::{Result, VsixHarvesterError};
use futures::stream::{self, StreamExt};
use marketplace::download_extension;

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
        if args.verbose {
            println!("Direct download mode for extension: {}", extension.to_id());
        }
        // Map architecture to target platform
        let target_platform = args
            .arch
            .as_deref()
            .and_then(Architecture::from_cli_arg)
            .and_then(|arch| arch.to_target_platform());

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
    let platforms = Architecture::available_architectures();

    // Process extensions for each platform
    for (platform_field, target_platform) in platforms {
        // Use reflection to get the field from the extensions struct
        let extensions_list = Architecture::get_extensions_list(platform_field, &extensions);

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
            let concurrent_downloads = if args.serial {
                1
            } else {
                MAX_CONCURRENT_DOWNLOADS
            };
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    process_extensions(&args).await
}
