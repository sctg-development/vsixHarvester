use std::fmt;
use std::str::FromStr;
use crate::error::VsixHarvesterError;
use crate::Extensions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    LinuxX64,
    LinuxArm64,
    DarwinX64,
    DarwinArm64,
    Win32X64,
    Win32Arm64,
    Universal,
}

impl Architecture {
    /// Get the target platform identifier for a specific architecture
    ///
    /// # Returns
    ///
    /// An Option containing the target platform identifier or None if the architecture is universal
    pub fn to_target_platform(&self) -> Option<&'static str> {
        match self {
            Self::LinuxX64 => Some("linux-x64"),
            Self::LinuxArm64 => Some("linux-arm64"),
            Self::DarwinX64 => Some("darwin-x64"),
            Self::DarwinArm64 => Some("darwin-arm64"),
            Self::Win32X64 => Some("win32-x64"),
            Self::Win32Arm64 => Some("win32-arm64"),
            Self::Universal => None,
        }
    }

    /// Get the platform field name for a specific architecture
    ///
    /// # Returns
    ///
    /// The platform field name
    pub fn to_field_name(&self) -> &'static str {
        match self {
            Self::LinuxX64 => "linux_x64",
            Self::LinuxArm64 => "linux_arm64",
            Self::DarwinX64 => "darwin_x64",
            Self::DarwinArm64 => "darwin_arm64",
            Self::Win32X64 => "win32_x64",
            Self::Win32Arm64 => "win32_arm64",
            Self::Universal => "universal",
        }
    }

    /// Get all available architectures
    ///
    /// # Returns
    ///
    /// A vector of all available architectures
    #[allow(dead_code)]
    pub fn all() -> Vec<Self> {
        vec![
            Self::Universal,
            Self::LinuxX64,
            Self::LinuxArm64, 
            Self::DarwinX64,
            Self::DarwinArm64,
            Self::Win32X64,
            Self::Win32Arm64,
        ]
    }

    /// Get available architectures with their target platform identifiers
    ///
    /// # Returns
    ///
    /// An array of tuples containing the platform field name and the target platform identifier
    pub fn available_architectures() -> [(&'static str, Option<&'static str>); 7] {
        [
            ("universal", None),
            ("linux_x64", Some("linux-x64")),
            ("linux_arm64", Some("linux-arm64")),
            ("darwin_x64", Some("darwin-x64")),
            ("darwin_arm64", Some("darwin-arm64")),
            ("win32_x64", Some("win32-x64")),
            ("win32_arm64", Some("win32-arm64")),
        ]
    }

    /// Get extensions list for a specific platform
    /// 
    /// # Arguments
    /// 
    /// * `platform_field` - The platform field name (e.g., "linux_x64")
    /// * `extensions` - The extensions structure
    /// 
    /// # Returns
    /// 
    /// An Option containing a reference to the extensions list for the given platform
    pub fn get_extensions_list<'a>(platform_field: &str, extensions: &'a Extensions) -> Option<&'a Vec<String>> {
        match platform_field {
            "universal" => extensions.universal.as_ref(),
            "linux_x64" => extensions.linux_x64.as_ref(),
            "linux_arm64" => extensions.linux_arm64.as_ref(),
            "darwin_x64" => extensions.darwin_x64.as_ref(),
            "darwin_arm64" => extensions.darwin_arm64.as_ref(),
            "win32_x64" => extensions.win32_x64.as_ref(),
            "win32_arm64" => extensions.win32_arm64.as_ref(),
            _ => None,
        }
    }

    /// Get the architecture from a CLI argument
    ///
    /// # Arguments
    ///
    /// * `arg` - The CLI argument
    ///
    /// # Returns
    ///
    /// An Option containing the architecture or None if the argument is invalid
    pub fn from_cli_arg(arg: &str) -> Option<Self> {
        Self::from_str(arg).ok()
    }
}

impl FromStr for Architecture {
    type Err = VsixHarvesterError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "linux_x64" => Ok(Self::LinuxX64),
            "linux_arm64" => Ok(Self::LinuxArm64),
            "darwin_x64" => Ok(Self::DarwinX64),
            "darwin_arm64" => Ok(Self::DarwinArm64),
            "win32_x64" => Ok(Self::Win32X64),
            "win32_arm64" => Ok(Self::Win32Arm64),
            "universal" => Ok(Self::Universal),
            _ => Err(VsixHarvesterError::InvalidArchitecture(s.to_string())),
        }
    }
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_field_name())
    }
}