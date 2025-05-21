use crate::config::{API_URL, MARKETPLACE_API_VERSION, MARKETPLACE_URL, USER_AGENT};
use crate::error::Result;
use crate::error::VsixHarvesterError;
use crate::extension::Extension;
use crate::types::MarketplaceResponse;
use log::{debug, error, info};
use serde::de;
use serde_json::json;
use std::fs;
use std::path::Path;

use bitflags::bitflags;

bitflags! {
    /// Flags that control what data is included in the marketplace API response
    pub struct Flags: u32 {
        /// None is used to retrieve only the basic extension details.
        const NONE = 0x0;

        /// IncludeVersions will return version information for extensions returned
        const INCLUDE_VERSIONS = 0x1;

        /// IncludeFiles will return information about which files were found
        /// within the extension that were stored independent of the manifest.
        /// When asking for files, versions will be included as well since files
        /// are returned as a property of the versions.
        /// These files can be retrieved using the path to the file without
        /// requiring the entire manifest be downloaded.
        const INCLUDE_FILES = 0x2;

        /// Include the Categories and Tags that were added to the extension definition.
        const INCLUDE_CATEGORY_AND_TAGS = 0x4;

        /// Include the details about which accounts the extension has been shared
        /// with if the extension is a private extension.
        const INCLUDE_SHARED_ACCOUNTS = 0x8;

        /// Include properties associated with versions of the extension
        const INCLUDE_VERSION_PROPERTIES = 0x10;

        /// Excluding non-validated extensions will remove any extension versions that
        /// either are in the process of being validated or have failed validation.
        const EXCLUDE_NON_VALIDATED = 0x20;

        /// Include the set of installation targets the extension has requested.
        const INCLUDE_INSTALLATION_TARGETS = 0x40;

        /// Include the base uri for assets of this extension
        const INCLUDE_ASSET_URI = 0x80;

        /// Include the statistics associated with this extension
        const INCLUDE_STATISTICS = 0x100;

        /// When retrieving versions from a query, only include the latest
        /// version of the extensions that matched. This is useful when the
        /// caller doesn't need all the published versions. It will save a
        /// significant size in the returned payload.
        const INCLUDE_LATEST_VERSION_ONLY = 0x200;

        /// The Unpublished extension flag indicates that the extension can't be installed/downloaded.
        /// Users who have installed such an extension can continue to use the extension.
        const UNPUBLISHED = 0x1000;

        /// Include the details if an extension is in conflict list or not
        const INCLUDE_NAME_CONFLICT_INFO = 0x8000;
    }
}

impl Flags {
    /// Creates the standard flags combination used for extension downloads
    pub fn standard() -> Self {
        // 914 (decimal) = 0x392 (hex) =
        // INCLUDE_VERSIONS | INCLUDE_FILES | INCLUDE_ASSET_URI | INCLUDE_STATISTICS | INCLUDE_LATEST_VERSION_ONLY
        // 0x1 | 0x2 | 0x80 | 0x100 | 0x200 = 0x392 (914)
        Flags::INCLUDE_VERSIONS
            | Flags::INCLUDE_FILES
            | Flags::INCLUDE_ASSET_URI
            | Flags::INCLUDE_STATISTICS
            | Flags::INCLUDE_LATEST_VERSION_ONLY
            | Flags::INCLUDE_VERSION_PROPERTIES
    }
    pub fn all_versions() -> Self {
        Flags::INCLUDE_VERSIONS | Flags::INCLUDE_FILES | Flags::INCLUDE_VERSION_PROPERTIES
    }
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
/// * `engine_version` - Optional, the engine to be compatible with
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
    engine_version: Option<&str>,
    allow_pre_release: bool,
) -> Result<()> {
    info!("Progress in extension: {}", extension.to_id());

    // Get latest version
    let version =
        get_extension_version(extension.clone(), proxy, engine_version, allow_pre_release).await?;
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
/// * `engine_version` - Optional engine version to filter by compatibility
/// * `verbose` - Whether to print verbose output
///
/// # Returns
///
/// A Result containing the version or an error that occurreds
pub async fn get_extension_version(
    extension: Extension<'_>,
    proxy: Option<&str>,
    engine_version: Option<&str>,
    allow_pre_release: bool,
) -> std::result::Result<String, VsixHarvesterError> {
    let api_url = API_URL;

    let (flags, str_engine_version) = if engine_version.is_some() {
        (Flags::all_versions().bits(), engine_version.unwrap())
    } else {
        (Flags::standard().bits(), "")
    };
    let payload = json!({
        "filters": [{
            "criteria": [
                {"filterType": 7, "value": format!("{}.{}", extension.publisher, extension.name)}
            ]
        }],
        "flags": flags
    });
    debug!("Using search payload: {}", payload);

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

    let json_body = resp.text().await?;

    let resp_json_result: std::result::Result<MarketplaceResponse, serde_json::Error> =
        serde_json::from_str(json_body.as_str());
    // If RUST_LOG is set to debug save the JSON response to a temporary file and display the path
    if std::env::var("RUST_LOG").is_ok_and(|v| v == "debug") {
        let temp_file_path = format!("./vsix_harvester_{}.json", extension.to_id());
        fs::write(&temp_file_path, &json_body)?;
        debug!("Saved JSON response to {}", temp_file_path);
    }
    if resp_json_result.is_err() {
        error!("Failed to parse JSON response");
        debug!("JSON was:\n{}", json_body.as_str());
        return Err(VsixHarvesterError::JsonError(
            resp_json_result.err().unwrap(),
        ));
    }
    let resp_json = resp_json_result.unwrap();
    debug!(
        "Got {} version results",
        resp_json.results[0].extensions[0].versions.len()
    );

    let versions = if engine_version.is_some() {
        resp_json.results[0].extensions[0]
            .get_compatible_versions(str_engine_version, allow_pre_release)
    } else {
        resp_json.results[0].extensions[0].versions.iter().collect()
    };

    let version = if engine_version.is_some() && !versions.is_empty() {
        // Debug the versions
        debug!(
            "Got {} version compatible with engine {}",
            versions.len(),
            str_engine_version
        );
        for current_version in versions.iter() {
            debug!(
                " - Version: {} Engine: {} PreRelease: {}",
                current_version.version,
                current_version
                    .get_vscode_engine_version()
                    .unwrap_or("None".to_string()),
                current_version
                    .get_vscode_prerelease()
                    .unwrap_or("false".to_string())
            );
        }

        versions[0].version.clone()
    } else {
        debug!("Could not find compatible version, using latest");
        resp_json.results[0].extensions[0].versions[0]
            .version
            .clone()
    };

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
