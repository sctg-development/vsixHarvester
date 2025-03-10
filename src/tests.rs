use crate::marketplace::{build_download_url_and_file_path, get_extension_version};
use crate::{
    create_directory_if_not_exists, download_extension, process_extensions, Args, Extension,
};
use std::fs;
use tempfile::TempDir;

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

#[test]
fn test_extension_to_id() {
    let ext = Extension {
        publisher: "publisher",
        name: "name",
    };
    assert_eq!(ext.to_id(), "publisher.name");
}

#[test]
fn test_build_download_url_and_file_path() {
    let ext = Extension {
        publisher: "publisher",
        name: "name",
    };
    let version = "1.0.0";
    let destination = "./extensions";
    let (download_url, file_path) =
        build_download_url_and_file_path(ext, version, destination, None);
    assert_eq!(download_url, "https://marketplace.visualstudio.com/_apis/public/gallery/publishers/publisher/vsextensions/name/1.0.0/vspackage");
    assert_eq!(file_path, "./extensions/publisher.name-1.0.0.vsix");
}

#[test]
fn test_get_extension_version() {
    let ext = Extension {
        publisher: "golang",
        name: "Go",
    };
    let version = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(get_extension_version(ext, None, None))
        .unwrap();
    assert!(!version.is_empty());
}

#[test]
fn test_create_directory_if_not_exists() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();
    let result = create_directory_if_not_exists(path);
    assert!(result.is_ok());
}

#[test]
fn test_create_directory_if_not_exists_already_exists() {
    let result = create_directory_if_not_exists("./src");
    assert!(result.is_ok());
}

#[test]
fn test_create_directory_if_not_exists_invalid_path() {
    let result = create_directory_if_not_exists("/invalid/path");
    assert!(result.is_err());
}

#[test]
fn test_download_extension_without_arch() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();

    let ext = Extension {
        publisher: "golang",
        name: "Go",
    };
    let destination = path;
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(download_extension(
            ext,
            destination,
            false,
            None,
            None,
            None,
        ));
    assert!(result.is_ok());
}

#[test]
fn test_download_extension_with_arch() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();
    let ext = Extension {
        publisher: "ms-python",
        name: "python",
    };
    let destination = path;
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(download_extension(
            ext,
            destination,
            false,
            None,
            Some("linux-x64"),
            None,
        ));
    assert!(result.is_ok());
    // Check that the extension has been downloaded by looking for files with specific patterns

    let python_exists = fs::read_dir(path)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .any(|entry| {
            let file_name = entry.file_name().into_string().unwrap_or_default();
            file_name.starts_with("ms-python.python-")
                && file_name.contains("linux-x64")
                && file_name.ends_with(".vsix")
        });
    assert!(
        python_exists,
        "python extension was not downloaded with linux-x64 target"
    );
}

/// Test the download of some extensions
#[test]
fn test_download_extensions() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();
    create_directory_if_not_exists(path).unwrap();
    // Create a test extensions.json file
    let json = r#"{
"universal": [
    "golang.Go"
],
"linux_x64": [
    "rust-lang.rust-analyzer"
],
"linux_arm64": [
    "ms-python.python"
]
}"#;
    fs::write(format!("{}/test_extensions.json", path), json).unwrap();

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(async {
        let args = Args {
            input: String::from(format!("{}/test_extensions.json", path)),
            destination: String::from(path),
            no_cache: true,
            proxy: None,
            verbose: true,
            download: None,
            arch: None,
            serial: true,
            engine_version: None,
        };

        process_extensions(&args).await
    });

    assert!(result.is_ok());
    // check that the 3 extensions have been downloaded using wildcard in the file name

    // Check that the extensions have been downloaded by looking for files with specific patterns
    let entries = fs::read_dir(path).unwrap();

    // Check for golang.Go
    let golang_exists = entries.filter_map(std::result::Result::ok).any(|entry| {
        let file_name = entry.file_name().into_string().unwrap_or_default();
        file_name.starts_with("golang.Go-") && file_name.ends_with(".vsix")
    });

    // Check for rust-analyzer
    let rust_analyzer_exists = fs::read_dir(path)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .any(|entry| {
            let file_name = entry.file_name().into_string().unwrap_or_default();
            file_name.starts_with("rust-lang.rust-analyzer-")
                && file_name.contains("linux-x64")
                && file_name.ends_with(".vsix")
        });

    // Check for python
    let python_exists = fs::read_dir(path)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .any(|entry| {
            let file_name = entry.file_name().into_string().unwrap_or_default();
            file_name.starts_with("ms-python.python-")
                && file_name.contains("linux-arm64")
                && file_name.ends_with(".vsix")
        });
    assert!(golang_exists, "golang.Go extension was not downloaded");
    assert!(
        rust_analyzer_exists,
        "rust-analyzer extension was not downloaded with linux-x64 target"
    );
    assert!(
        python_exists,
        "python extension was not downloaded with linux-arm64 target"
    );
}

/// Test the download of some extensions for a specific engine
#[test]
fn test_download_extensions_for_specific_engine() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();
    create_directory_if_not_exists(path).unwrap();
    // Create a test extensions.json file
    let json = r#"{
"universal": [
"golang.Go"
],
"linux_arm64": [
"rust-lang.rust-analyzer"
],
"linux_x64": [
"ms-python.python"
]
}"#;

    fs::write(format!("{}/test_extensions.json", path), json).unwrap();

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(async {
        let args = Args {
            input: String::from(format!("{}/test_extensions.json", path)),
            destination: String::from(path),
            no_cache: true,
            proxy: None,
            verbose: true,
            download: None,
            arch: None,
            serial: true,
            engine_version: Some(String::from("1.97.0")),
        };

        process_extensions(&args).await
    });

    assert!(result.is_ok());
    // check that the 3 extensions have been downloaded using wildcard in the file name

    // Check that the extensions have been downloaded by looking for files with specific patterns
    let entries = fs::read_dir(path).unwrap();

    // Check for golang.Go
    let golang_exists = entries.filter_map(std::result::Result::ok).any(|entry| {
        let file_name = entry.file_name().into_string().unwrap_or_default();
        file_name.starts_with("golang.Go-") && file_name.ends_with(".vsix")
    });

    // Check for rust-analyzer
    let rust_analyzer_exists = fs::read_dir(path)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .any(|entry| {
            let file_name = entry.file_name().into_string().unwrap_or_default();
            file_name.starts_with("rust-lang.rust-analyzer-")
                && file_name.contains("linux-arm64")
                && file_name.ends_with(".vsix")
        });

    // Check for python
    let python_exists = fs::read_dir(path)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .any(|entry| {
            let file_name = entry.file_name().into_string().unwrap_or_default();
            file_name.starts_with("ms-python.python-")
                && file_name.contains("linux-x64")
                && file_name.ends_with(".vsix")
        });
    assert!(golang_exists, "golang.Go extension was not downloaded");
    assert!(
        rust_analyzer_exists,
        "rust-analyzer extension was not downloaded with linux-x64 target"
    );
    assert!(
        python_exists,
        "python extension was not downloaded with linux-arm64 target"
    );
}
