use crate::config::{GITHUB_API_URL, GITHUB_RELEASES_URL, USER_AGENT, VERSION};
use crate::error::{Result, VsixHarvesterError};
use log::info;
use reqwest;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Response structure for GitHub API latest release
#[derive(serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

/// Get the binary filename for the current platform
fn get_binary_filename(version: &str) -> Result<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let filename = match (os, arch) {
        ("linux", "x86_64") => format!("vsixHarvester_linux_amd64_static_{version}"),
        ("linux", "aarch64") => format!("vsixHarvester_linux_arm64_static_{version}"),
        ("linux", "arm") => format!("vsixHarvester_linux_armhf_static_{version}"),
        ("macos", "x86_64") => format!("vsixHarvester_macos_amd64_{version}"),
        ("macos", "aarch64") => format!("vsixHarvester_macos_arm64_{version}"),
        ("windows", "x86_64") => format!("vsixHarvester_windows_amd64_{version}.exe"),
        _ => {
            return Err(VsixHarvesterError::UpgradeError(format!(
                "Unsupported platform: {os}/{arch}"
            )));
        }
    };

    Ok(filename)
}

/// Build the download URL for the current platform
fn build_download_url(version: &str) -> Result<String> {
    let filename = get_binary_filename(version)?;
    let url = format!("{}/{}/{}", GITHUB_RELEASES_URL, version, filename);
    Ok(url)
}

/// Fetch the latest release version from GitHub API
async fn fetch_latest_version(client: &reqwest::Client) -> Result<String> {
    let resp = client
        .get(GITHUB_API_URL)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(VsixHarvesterError::UpgradeError(format!(
            "GitHub API request failed with status: {}",
            resp.status()
        )));
    }

    let release: GitHubRelease = resp.json().await?;
    let version = release.tag_name.trim_start_matches('v').to_string();
    Ok(version)
}

/// Perform the upgrade: download latest binary and replace current executable
pub async fn upgrade(proxy: Option<&str>, retry_count: u32) -> Result<()> {
    info!("Checking for updates...");

    // Build HTTP client with optional proxy
    let client_builder = reqwest::Client::builder();
    let client = if let Some(proxy_url) = proxy {
        info!("Using proxy for upgrade: {}", proxy_url);
        let proxy = reqwest::Proxy::all(proxy_url)?;
        client_builder.proxy(proxy).build()?
    } else {
        client_builder.build()?
    };

    // Fetch latest version
    let latest_version = fetch_latest_version(&client).await?;
    info!("Latest version available: {}", latest_version);
    info!("Current version: {}", VERSION);

    // Compare versions
    if latest_version == VERSION {
        info!("Already up to date (version {})", VERSION);
        return Ok(());
    }

    info!("New version {} available, downloading...", latest_version);

    // Build download URL
    let download_url = build_download_url(&latest_version)?;
    info!("Download URL: {}", download_url);

    // Download the new binary with retry logic
    let binary_data = download_with_retry(&client, &download_url, retry_count).await?;

    // Get current executable path
    let current_exe = std::env::current_exe()
        .map_err(|e| VsixHarvesterError::UpgradeError(format!("Failed to get current executable path: {}", e)))?;

    info!("Current executable: {}", current_exe.display());

    // Create temp directory and write new binary
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path().join("vsixHarvester_new");

    // On Windows, we need .exe extension
    #[cfg(windows)]
    let temp_path = temp_path.with_extension("exe");

    fs::write(&temp_path, &binary_data)?;
    info!("Downloaded binary to: {}", temp_path.display());

    // Make it executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_path, perms)?;
    }

    // Replace current executable
    replace_binary(&temp_path, &current_exe)?;

    info!("Successfully upgraded to version {}", latest_version);
    Ok(())
}

/// Download with exponential backoff retry
async fn download_with_retry(
    client: &reqwest::Client,
    url: &str,
    retry_count: u32,
) -> Result<Vec<u8>> {
    let mut attempts = 0;
    let max_attempts = retry_count + 1;

    loop {
        attempts += 1;
        let resp = client.get(url).send().await;

        match resp {
            Ok(response) => {
                if response.status().is_success() {
                    let bytes = response.bytes().await?;
                    return Ok(bytes.to_vec());
                }
                if attempts < max_attempts {
                    info!(
                        "Download failed with status {}, retrying (attempt {}/{})",
                        response.status(),
                        attempts,
                        max_attempts
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempts))).await;
                    continue;
                }
                return Err(VsixHarvesterError::UpgradeError(format!(
                    "Download failed with status: {}",
                    response.status()
                )));
            }
            Err(e) => {
                if attempts < max_attempts {
                    info!(
                        "Download error: {}, retrying (attempt {}/{})",
                        e, attempts, max_attempts
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempts))).await;
                    continue;
                }
                return Err(VsixHarvesterError::UpgradeError(format!("Download failed: {}", e)));
            }
        }
    }
}

/// Replace the current binary with the new one
fn replace_binary(new_path: &Path, current_path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        // On Unix, we can replace a running binary
        // The old inode remains in memory until the process exits
        fs::rename(new_path, current_path)?;
        info!("Binary replaced successfully");
    }

    #[cfg(windows)]
    {
        // On Windows, we cannot replace a running binary
        // We copy the new binary and print instructions
        fs::copy(new_path, current_path)?;
        info!("Binary copied. Please restart the application to use the new version.");
        info!("Note: On Windows, you may need to manually replace the binary if the copy fails due to file locking.");
    }

    Ok(())
}