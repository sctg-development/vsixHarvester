pub use serde::Deserialize;

use crate::error::VsixHarvesterError;
#[derive(Clone)]
pub struct Extension<'a> {
    pub publisher: &'a str,
    pub name: &'a str,
}

impl<'a> Extension<'a> {
    pub fn from_id(id: &'a str) -> std::result::Result<Self, VsixHarvesterError> {
        let parts: Vec<&str> = id.split('.').collect();
        if parts.len() != 2 {
            return Err(VsixHarvesterError::InvalidExtensionId(id.to_string()));
        }
        Ok(Self {
            publisher: parts[0],
            name: parts[1],
        })
    }

    pub fn to_id(&self) -> String {
        format!("{}.{}", self.publisher, self.name)
    }

    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        format!("Publisher: {}, Name: {}", self.publisher, self.name)
    }
}

#[derive(Deserialize)]
pub struct Extensions {
    pub universal: Option<Vec<String>>,
    pub linux_x64: Option<Vec<String>>,
    pub linux_arm64: Option<Vec<String>>,
    pub darwin_x64: Option<Vec<String>>,
    pub darwin_arm64: Option<Vec<String>>,
    pub win32_x64: Option<Vec<String>>,
    pub win32_arm64: Option<Vec<String>>,
}