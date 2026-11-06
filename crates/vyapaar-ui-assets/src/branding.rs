//! Brand identity constants for the Vyapaar workstation.

use serde::{Deserialize, Serialize};

/// Product/brand name.
pub const BRAND_NAME: &str = "व्यापार Vyapaar";

/// Short tagline used in the shell title and home header.
pub const TAGLINE: &str = "Indian Equities Algo Workstation";

/// Semantic version of the workstation UI.
pub const UI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Brand identity bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branding {
    /// Brand name.
    pub name: String,
    /// Tagline.
    pub tagline: String,
    /// Version string.
    pub version: String,
}

impl Branding {
    /// Create a new branding bundle.
    pub fn new(name: impl Into<String>, tagline: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tagline: tagline.into(),
            version: version.into(),
        }
    }

    /// The default Vyapaar branding.
    pub fn default_brand() -> Self {
        Self::new(BRAND_NAME, TAGLINE, UI_VERSION)
    }
}

/// Brand name helper.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrandName {
    /// Full brand string.
    pub full: String,
}

impl BrandName {
    /// Create a new brand name.
    pub fn new(full: impl Into<String>) -> Self {
        Self { full: full.into() }
    }

    /// The canonical Vyapaar brand name.
    pub fn vyapaar() -> Self {
        Self::new(BRAND_NAME)
    }
}
