#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ListVariants)]
/// Supported architectures
pub enum Architecture {
    /// 64 bit Arm
    Arm64,
    /// 64 bit
    #[serde(rename = "64bit")]
    X64,
    #[serde(rename = "32bit")]
    /// 32 bit
    X86,
}

impl FromStr for Architecture {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "64bit" => Self::X64,
            "32bit" => Self::X86,
            "arm64" => Self::Arm64,
            _ => return Err(Error::UnsupportedArchitecture),
        })
    }
}

impl Architecture {
    /// Get the architecture of the current environment
    pub const ARCH: Self = Self::X64;

    #[must_use]
    /// Get the architecture of the current environment
    ///
    /// # Panics
    /// - Unsupported environment
    pub const fn from_env() -> Self {
        if cfg!(target_arch = "x86_64") {
            Self::X64
        } else if cfg!(target_arch = "x86") {
            Self::X86
        } else if cfg!(target_arch = "aarch64") {
            Self::Arm64
        } else {
            panic!("Unsupported architecture")
        }
    }

    #[must_use]
    /// Get the architecture of a given Scoop string
    ///
    /// # Panics
    /// - Unsupported environment
    pub fn from_scoop_string(string: &str) -> Self {
        match string {
            "64bit" => Self::X64,
            "32bit" => Self::X86,
            "arm64" => Self::Arm64,
            _ => panic!("Unsupported architecture"),
        }
    }
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arm64 => write!(f, "arm64"),
            Self::X64 => write!(f, "64bit"),
            Self::X86 => write!(f, "32bit"),
        }
    }
}

impl Default for Architecture {
    fn default() -> Self {
        Self::from_env()
    }
}
