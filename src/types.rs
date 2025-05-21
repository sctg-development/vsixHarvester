use log::debug;
use serde::{Deserialize, Serialize};

/// Response from the VS Code marketplace API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketplaceResponse {
    pub results: Vec<ResultItem>,
}

/// A result item from the VS Code marketplace API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResultItem {
    pub extensions: Vec<Extension>,
    #[serde(rename = "pagingToken")]
    pub paging_token: Option<String>,
    #[serde(rename = "resultMetadata")]
    pub result_metadata: Vec<ResultMetadata>,
}

/// An extension from the VS Code marketplace
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Extension {
    pub publisher: Publisher,
    #[serde(rename = "extensionId")]
    pub extension_id: String,
    #[serde(rename = "extensionName")]
    pub extension_name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub flags: String,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
    #[serde(rename = "publishedDate")]
    pub published_date: String,
    #[serde(rename = "releaseDate")]
    pub release_date: String,
    #[serde(rename = "shortDescription")]
    pub short_description: String,
    pub versions: Vec<Version>,
    #[serde(rename = "deploymentType")]
    pub deployment_type: i32,
}

/// Publisher information for an extension
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Publisher {
    #[serde(rename = "publisherId")]
    pub publisher_id: String,
    #[serde(rename = "publisherName")]
    pub publisher_name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub flags: String,
    pub domain: Option<String>,
    #[serde(rename = "isDomainVerified")]
    pub is_domain_verified: bool,
}

/// Version information for an extension
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Version {
    pub version: String,
    pub flags: String,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
    pub files: Vec<File>,
    pub properties: Option<Vec<Property>>,
    #[serde(rename = "assetUri")]
    pub asset_uri: String,
    #[serde(rename = "fallbackAssetUri")]
    pub fallback_asset_uri: String,
}

/// File information for an extension version
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    #[serde(rename = "assetType")]
    pub asset_type: String,
    pub source: String,
}

/// Property information for an extension version
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Property {
    pub key: String,
    pub value: String,
}

/// Metadata for a result
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResultMetadata {
    #[serde(rename = "metadataType")]
    pub metadata_type: String,
    #[serde(rename = "metadataItems")]
    pub metadata_items: Vec<MetadataItem>,
}

/// Item in result metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataItem {
    pub name: String,
    pub count: i32,
}

/// Helper methods for the types
impl Extension {
    /// Gets the VSIX package URL for the latest version of the extension
    #[allow(dead_code)]
    pub fn get_latest_vsix_url(&self) -> Option<String> {
        if self.versions.is_empty() {
            return None;
        }

        let latest_version = &self.versions[0];
        latest_version.get_vsix_url()
    }

    /// Gets the identifier of the extension in the format "publisher.name"
    #[allow(dead_code)]
    pub fn get_identifier(&self) -> String {
        format!("{}.{}", self.publisher.publisher_name, self.extension_name)
    }

    /// Gets versions compatible with a specific VS Code engine version
    ///
    /// # Arguments
    ///
    /// * `engine` - The VS Code engine version to check compatibility with (e.g., "1.97.0")
    ///
    /// # Returns
    ///
    /// A vector of references to compatible versions
    pub fn get_compatible_versions<'a>(&'a self, engine: &str) -> Vec<&'a Version> {
        self.versions
            .iter()
            .filter(|version| {
                version
                    .get_vscode_engine_version()
                    .map_or(false, |req| is_compatible(req.as_str(), engine))
            })
            .collect()
    }

    /// Gets only the non prerelease versions
    ///
    /// # Returns
    ///
    /// A vector of references to non prerelease versions
    pub fn get_non_prerelease_versions<'a>(&'a self) -> Vec<&'a Version> {
        self.versions
            .iter()
            .filter(|version| {
                version
                    .get_vscode_prerelease()
                    .map_or(false, |req| req.is_empty() || req != "true")
            })
            .collect()
    }
}

impl Version {
    /// Gets the VSIX package URL for this version
    #[allow(dead_code)]
    pub fn get_vsix_url(&self) -> Option<String> {
        self.files
            .iter()
            .find(|file| file.asset_type == "Microsoft.VisualStudio.Services.VSIXPackage")
            .map(|file| file.source.clone())
    }
    /// Filter all the properties to get the one with the key "Microsoft.VisualStudio.Code.Engine"
    pub fn get_vscode_engine_version(&self) -> Option<String> {
        self.properties
            .clone()
            .unwrap_or_default()
            .iter()
            .find(|property| property.key == "Microsoft.VisualStudio.Code.Engine")
            .map(|property| property.value.clone())
    }
    /// Filter all the properties to get one with the key "Microsoft.VisualStudio.Code.PreRelease"
    pub fn get_vscode_prerelease(&self) -> Option<String> {
        self.properties
            .clone()
            .unwrap_or_default()
            .iter()
            .find(|property| property.key == "Microsoft.VisualStudio.Code.PreRelease")
            .map(|property| property.value.clone())
    }
}

/// Checks if the required version is compatible with the provided engine version
///
/// # Arguments
///
/// * `requirement` - The version requirement (e.g., "^1.97.0", ">=1.96.0")
/// * `engine_version` - The engine version to check against (e.g., "1.97.0")
///
/// # Returns
///
/// `true` if compatible, `false` otherwise
fn is_compatible(requirement: &str, engine_version: &str) -> bool {
    // Simple version: just check if the major.minor.patch version matches
    // For a more comprehensive solution, a proper semver library would be better

    // Handle caret (^) requirements: Compatible with the specified major.minor version
    if let Some(req_version) = requirement.strip_prefix('^') {
        let req_parts: Vec<&str> = req_version.split('.').collect();
        let engine_parts: Vec<&str> = engine_version.split('.').collect();

        // For caret, major version must match
        if req_parts.len() >= 2 && engine_parts.len() >= 2 && req_parts[0] == engine_parts[0] {
            return req_parts[1] == engine_parts[1];
        }
    }
    // Handle greater-than-or-equal (>=) requirements
    else if let Some(req_version) = requirement.strip_prefix(">=") {
        return compare_versions(engine_version, req_version.trim()) >= 0;
    }
    // Handle exact version match (no prefix)
    else if !requirement.contains(|c: char| !c.is_digit(10) && c != '.') {
        return requirement == engine_version;
    }
    // Handle simple contains check as a fallback
    else {
        return engine_version.contains(requirement);
    }

    false
}

/// Compare two version strings
///
/// # Arguments
///
/// * `version_a` - First version string (e.g., "1.97.0")
/// * `version_b` - Second version string (e.g., "1.96.0")
///
/// # Returns
///
/// * `1` if version_a > version_b
/// * `0` if version_a == version_b
/// * `-1` if version_a < version_b
fn compare_versions(version_a: &str, version_b: &str) -> i32 {
    let parts_a: Vec<u32> = version_a
        .split('.')
        .filter_map(|s| s.parse::<u32>().ok())
        .collect();
    let parts_b: Vec<u32> = version_b
        .split('.')
        .filter_map(|s| s.parse::<u32>().ok())
        .collect();

    let max_len = std::cmp::max(parts_a.len(), parts_b.len());

    for i in 0..max_len {
        let a = parts_a.get(i).copied().unwrap_or(0);
        let b = parts_b.get(i).copied().unwrap_or(0);

        if a > b {
            return 1;
        } else if a < b {
            return -1;
        }
    }

    0
}

/// Helper function to parse a marketplace response from a JSON string
#[allow(dead_code)]
pub fn parse_marketplace_response(json: &str) -> Result<MarketplaceResponse, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_marketplace_response() {
        // Sample response JSON (shortened for brevity)
        let json = r#"{
            "results": [
                {
                    "extensions": [
                        {
                            "publisher": {
                                "publisherId": "7c1c19cd-78eb-4dfb-8999-99caf7679002",
                                "publisherName": "GitHub",
                                "displayName": "GitHub",
                                "flags": "verified",
                                "domain": "https://github.com",
                                "isDomainVerified": true
                            },
                            "extensionId": "23c4aeee-f844-43cd-b53e-1113e483f1a6",
                            "extensionName": "copilot",
                            "displayName": "GitHub Copilot",
                            "flags": "validated, public, trial",
                            "lastUpdated": "2025-03-09T04:19:46.193Z",
                            "publishedDate": "2021-06-29T14:26:17.88Z",
                            "releaseDate": "2021-06-29T14:26:17.88Z",
                            "shortDescription": "Your AI pair programmer",
                            "versions": [
                                {
                                    "version": "1.280.1421",
                                    "flags": "validated",
                                    "lastUpdated": "2025-03-09T04:19:46.193Z",
                                    "files": [
                                        {
                                            "assetType": "Microsoft.VisualStudio.Services.VSIXPackage",
                                            "source": "https://GitHub.gallerycdn.vsassets.io/extensions/github/copilot/1.280.1421/1741493793091/Microsoft.VisualStudio.Services.VSIXPackage"
                                        }
                                    ],
                                    "properties": [
                                        {
                                            "key": "Microsoft.VisualStudio.Code.Engine",
                                            "value": "^1.98.0"
                                        }
                                    ],
                                    "assetUri": "https://GitHub.gallerycdn.vsassets.io/extensions/github/copilot/1.280.1421/1741493793091",
                                    "fallbackAssetUri": "https://GitHub.gallery.vsassets.io/_apis/public/gallery/publisher/GitHub/extension/copilot/1.280.1421/assetbyname"
                                }
                            ],
                            "deploymentType": 0
                        }
                    ],
                    "pagingToken": null,
                    "resultMetadata": [
                        {
                            "metadataType": "ResultCount",
                            "metadataItems": [
                                {
                                    "name": "TotalCount",
                                    "count": 1
                                }
                            ]
                        }
                    ]
                }
            ]
        }"#;

        let response = parse_marketplace_response(json);
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].extensions.len(), 1);

        let extension = &response.results[0].extensions[0];
        assert_eq!(extension.extension_name, "copilot");
        assert_eq!(extension.publisher.publisher_name, "GitHub");
        assert_eq!(extension.get_identifier(), "GitHub.copilot");

        assert!(extension.get_latest_vsix_url().is_some());
        let vsix_url = extension.get_latest_vsix_url().unwrap();
        assert!(vsix_url.contains("Microsoft.VisualStudio.Services.VSIXPackage"));
    }

    #[cfg(test)]
    #[test]
    fn test_get_compatible_versions() {
        // Créer une extension de test avec différentes versions
        let mut extension = Extension {
            publisher: Publisher {
                publisher_id: "test-id".to_string(),
                publisher_name: "test".to_string(),
                display_name: "Test".to_string(),
                flags: "".to_string(),
                domain: Some("".to_string()),
                is_domain_verified: false,
            },
            extension_id: "test-ext-id".to_string(),
            extension_name: "test-ext".to_string(),
            display_name: "Test Extension".to_string(),
            flags: "".to_string(),
            last_updated: "".to_string(),
            published_date: "".to_string(),
            release_date: "".to_string(),
            short_description: "".to_string(),
            versions: vec![],
            deployment_type: 0,
        };

        // Add versions with different engine requirements
        extension.versions.push(Version {
            version: "1.0.0".to_string(),
            flags: "".to_string(),
            last_updated: "".to_string(),
            files: vec![],
            properties: vec![Property {
                key: "Microsoft.VisualStudio.Code.Engine".to_string(),
                value: "^1.97.0".to_string(),
            }]
            .into(),
            asset_uri: "".to_string(),
            fallback_asset_uri: "".to_string(),
        });

        extension.versions.push(Version {
            version: "2.0.0".to_string(),
            flags: "".to_string(),
            last_updated: "".to_string(),
            files: vec![],
            properties: vec![Property {
                key: "Microsoft.VisualStudio.Code.Engine".to_string(),
                value: "^1.98.0".to_string(),
            }]
            .into(),
            asset_uri: "".to_string(),
            fallback_asset_uri: "".to_string(),
        });

        extension.versions.push(Version {
            version: "3.0.0".to_string(),
            flags: "".to_string(),
            last_updated: "".to_string(),
            files: vec![],
            properties: vec![Property {
                key: "Microsoft.VisualStudio.Code.Engine".to_string(),
                value: "^1.97.0".to_string(),
            }]
            .into(),
            asset_uri: "".to_string(),
            fallback_asset_uri: "".to_string(),
        });

        // Tester la fonction
        let v197 = extension.get_compatible_versions("1.97.0");
        assert_eq!(v197.len(), 2);
        assert_eq!(v197[0].version, "1.0.0");
        assert_eq!(v197[1].version, "3.0.0");

        let v198 = extension.get_compatible_versions("1.98.0");
        assert_eq!(v198.len(), 1);
        assert_eq!(v198[0].version, "2.0.0");
    }

    #[test]
    fn test_version_compatibility() {
        // Tests pour la fonction is_compatible
        assert!(is_compatible("^1.97.0", "1.97.0"));
        assert!(is_compatible("^1.97.0", "1.97.5"));
        assert!(!is_compatible("^1.97.0", "2.0.0"));
        assert!(!is_compatible("^1.97.0", "1.96.0"));

        assert!(is_compatible(">=1.96.0", "1.96.0"));
        assert!(is_compatible(">=1.96.0", "1.97.0"));
        assert!(!is_compatible(">=1.97.0", "1.96.0"));

        assert!(is_compatible("1.97.0", "1.97.0"));
        assert!(!is_compatible("1.97.0", "1.97.1"));
    }
}
