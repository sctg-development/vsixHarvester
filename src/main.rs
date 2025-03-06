use clap::Parser;
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use std::fs;
use std::path::Path;
use tokio;

#[derive(Parser)]
#[command(version = "0.2.1", about = "Download VSCode extensions for offline use")]
struct Args {
    /// Path to extensions.json
    #[arg(short, long, default_value = "./extensions.json")]
    input: String,

    /// Output directory
    #[arg(short, long, default_value = "./extensions")]
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

fn create_directory_if_not_exists(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(path);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Read extensions.json
    if args.verbose {
        println!("Attempting to read file: {}", &args.input);
    }
    let file_content = match fs::read_to_string(&args.input) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file {}: {}", &args.input, e);
            return Err(Box::new(e) as Box<dyn Error>);
        }
    };
    let extensions: Extensions = match serde_json::from_str(&file_content) {
        Ok(extensions) => extensions,
        Err(e) => {
            eprintln!("Failed to parse file {}: {}", &args.input, e);
            return Err(Box::new(e) as Box<dyn Error>);
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
            for extension in ext_list {
                if args.verbose {
                    println!("Attempting to download extension: {}", &extension);
                }
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
                    eprintln!("Error occurred when downloading {}: {}", extension, e);
                }
            }
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

    let download_url: String;
    let file_name: String;
    let file_path: String;
    if os_arch.is_some() {
        let target_platform = os_arch.unwrap();
        file_name = format!("{publisher}.{extension_name}-{version}@{target_platform}.vsix");
        file_path = format!("{}/{}", destination, file_name);
        download_url = format!(
            "https://marketplace.visualstudio.com/_apis/public/gallery/publishers/{publisher}/vsextensions/{extension_name}/{version}/vspackage?targetPlatform={target_platform}",
            publisher = publisher,
            extension_name = extension_name,
            version = version,
            target_platform = os_arch.unwrap()
        );
    } else {
        file_name = format!("{publisher}.{extension_name}-{version}.vsix");
        file_path = format!("{}/{}", destination, file_name);
        download_url = format!(
            "https://marketplace.visualstudio.com/_apis/public/gallery/publishers/{publisher}/vsextensions/{extension_name}/{version}/vspackage",
            publisher = publisher,
            extension_name = extension_name,
            version = version
        );
    }

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
