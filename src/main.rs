use clap::Parser;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::path::Path;
use tokio;

#[derive(Parser)]
struct Args {
    /// Path to extensions.json
    #[arg(short, long, default_value = "./.vscode/extensions.json")]
    input: String,

    /// Output directory
    #[arg(short, long, default_value = "./.vscode/extensions")]
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

    /// Specify OS architecture
    #[arg(long)]
    os_arch: Option<String>,
}

#[derive(Deserialize)]
struct Extensions {
    recommendations: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Read extensions.json
    let file_content = fs::read_to_string(&args.input)?;
    let extensions: Extensions = serde_json::from_str(&file_content)?;

    // Create output directory
    fs::create_dir_all(&args.destination)?;

    // Download each extensions
    for extension in extensions.recommendations {
        if let Err(e) = download_extension(
            &extension,
            &args.destination,
            args.no_cache,
            args.proxy.as_deref(),
            args.verbose,
            args.os_arch.as_deref(),
        )
        .await
        {
            eprintln!("Error has occored when downloading {}: {}", extension, e);
        }
    }

    Ok(())
}

async fn download_extension(
    extension: &str,
    destination: &str,
    no_cache: bool,
    proxy: Option<&str>,
    verbose: bool,
    os_arch: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Progress in extension: {}", extension);
    }

    let parts: Vec<&str> = extension.split('.').collect();
    if parts.len() != 2 {
        eprintln!("Invalid extension identifier: {}", extension);
        return Ok(());
    }
    let publisher = parts[0];
    let extension_name = parts[1];

    // Get latest version
    let version = get_extension_version(publisher, extension_name, proxy, verbose).await?;
    if verbose {
        println!("Latest version of {}: {}", extension, version);
    }

    // Create download url
    let target_platform = match os_arch {
        Some("darwin-x64") => "darwin-x64",
        Some("darwin-arm64") => "darwin-arm64",
        Some("win32-x64") => "win32-x64",
        Some("win32-arm64") => "win32-arm64",
        Some("linux-x64") => "linux-x64",
        Some("linux-arm64") => "linux-arm64",
        Some(other) => {
            eprintln!("Unsupported OS architecture: {}", other);
            return Ok(());
        }
        None => {
            eprintln!("OS architecture not specified.");
            return Ok(());
        }
    };

    let download_url = format!(
        "https://marketplace.visualstudio.com/_apis/public/gallery/publishers/{publisher}/vsextensions/{extension_name}/{version}/vspackage?targetPlatform={target_platform}",
        publisher = publisher,
        extension_name = extension_name,
        version = version,
        target_platform = target_platform
    );

    if verbose {
        println!("Download URL: {}", download_url);
    }

    // Make file path
    let file_name = format!("{publisher}.{extension_name}-{version}@{target_platform}.vsix");
    let file_path = format!("{}/{}", destination, file_name);

    // Check file already exists
    if !no_cache && Path::new(&file_path).exists() {
        if verbose {
            println!("file {} is already exists. Skip download.", file_path);
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
        client_builder.proxy(proxy).build()?
    } else {
        client_builder.build()?
    };

    // Download VSIX file
    if verbose {
        println!("Download form {}", download_url);
    }
    let resp = client.get(&download_url).send().await?;
    if !resp.status().is_success() {
        eprintln!("Fail download of {}", extension);
        return Err(Box::from("Fail download of VSIX"));
    }
    let vsix_content = resp.bytes().await?;

    // Save file
    fs::write(&file_path, &vsix_content)?;
    if verbose {
        println!("Saved in {}", file_path);
    }

    Ok(())
}

async fn get_extension_version(
    publisher: &str,
    extension_name: &str,
    proxy: Option<&str>,
    verbose: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let api_url = "https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery";

    let payload = json!({
        "filters": [{
            "criteria": [
                {"filterType": 7, "value": format!("{}.{}", publisher, extension_name)}
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
            publisher, extension_name
        );
    }
    let resp = client
        .post(api_url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json;api-version=3.0-preview.1")
        .header("User-Agent", "Offline VSIX/1.0")
        .json(&payload)
        .send()
        .await?;

    if !resp.status().is_success() {
        eprintln!("Failed query for Marketplace API");
        return Err(Box::from("Failed query for Marketplace API"));
    }

    let resp_json: serde_json::Value = resp.json().await?;

    // Extract version
    let version = resp_json["results"][0]["extensions"][0]["versions"][0]["version"]
        .as_str()
        .ok_or("Failed get extension version")?
        .to_string();

    Ok(version)
}
