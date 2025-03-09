use crate::config::{API_URL, MARKETPLACE_API_VERSION, MARKETPLACE_URL, USER_AGENT};
use crate::error::Result;
use crate::error::VsixHarvesterError;
use crate::extension::Extension;
use log::{debug, error, info};
use serde_json::json;
use std::fs;
use std::path::Path;

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
pub async fn download_extension(
    extension: Extension<'_>,
    destination: &str,
    no_cache: bool,
    proxy: Option<&str>,
    os_arch: Option<&str>,
) -> Result<()> {
    info!("Progress in extension: {}", extension.to_id());

    // Get latest version
    let version = get_extension_version(extension.clone(), proxy).await?;
    info!("Latest version of {}: {}", extension.to_id(), version);

    let (download_url, file_path) =
        build_download_url_and_file_path(extension.clone(), &version, destination, os_arch);

    debug!("Download URL: {}", download_url);

    // Make file path

    // Check file already exists
    if !no_cache && Path::new(&file_path).exists() {
        info!(
            "Skip download: File is already exists. File Name {}.",
            file_path
        );
        return Ok(());
    }

    // Create http client
    let client_builder = reqwest::Client::builder();
    let client = if let Some(proxy_url) = proxy {
        info!("Using proxy: {}", proxy_url);
        let proxy = reqwest::Proxy::all(proxy_url)?;
        client_builder.gzip(true).proxy(proxy).build()?
    } else {
        client_builder.gzip(true).build()?
    };

    // Download VSIX file
    info!("Download form {}", download_url);
    let resp = client
        .get(&download_url)
        .header(reqwest::header::ACCEPT_ENCODING, "gzip")
        .send()
        .await?;
    if !resp.status().is_success() {
        error!("Fail download of {}", extension.to_id());
        return Err(VsixHarvesterError::DownloadError(extension.to_id()));
    }

    let vsix_raw_content = resp.bytes().await?;

    // Save file
    fs::write(&file_path, &vsix_raw_content)?;
    info!("Saved in {}", file_path);

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
pub async fn get_extension_version(
    extension: Extension<'_>,
    proxy: Option<&str>,
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
        info!("Using proxy for API request: {}", proxy_url);
        let proxy = reqwest::Proxy::all(proxy_url)?;
        client_builder.proxy(proxy).build()?
    } else {
        client_builder.build()?
    };

    // Send POST request
    debug!(
        "Sending query for Marketplace API: {}.{}",
        extension.publisher, extension.name
    );
    let resp = client
        .post(api_url)
        .header("Content-Type", "application/json")
        .header(
            "Accept",
            format!("application/json;api-version={}", MARKETPLACE_API_VERSION),
        )
        .header("User-Agent", USER_AGENT)
        .json(&payload)
        .send()
        .await?;

    if !resp.status().is_success() {
        error!("Failed query for Marketplace API");
        return Err(VsixHarvesterError::ApiError(
            "Failed query for Marketplace API".to_string(),
        ));
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
pub fn build_download_url_and_file_path(
    extension: Extension<'_>,
    version: &str,
    destination: &str,
    os_arch: Option<&str>,
) -> (String, String) {
    let file_name: String;
    let file_path: String;
    let download_url: String;

    if let Some(target_platform) = os_arch {
        file_name = format!(
            "{}.{}-{version}@{}.vsix",
            extension.publisher, extension.name, target_platform
        );
        file_path = format!("{}/{}", destination, file_name);
        download_url = format!(
            "{}/{}/vsextensions/{}/{}/vspackage?targetPlatform={}",
            MARKETPLACE_URL, extension.publisher, extension.name, version, target_platform
        );
    } else {
        file_name = format!(
            "{}.{}-{}.vsix",
            extension.publisher, extension.name, version
        );
        file_path = format!("{}/{}", destination, file_name);
        download_url = format!(
            "{}/{}/vsextensions/{}/{}/vspackage",
            MARKETPLACE_URL, extension.publisher, extension.name, version
        );
    }

    (download_url, file_path)
}
