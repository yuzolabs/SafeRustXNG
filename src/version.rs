//! Version information, mirroring `searx/version.py`.

/// The crate version, taken from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Version string shown in the UI ("SafeRustXNG x.y.z").
pub fn version_string() -> String {
    format!("SafeRustXNG {VERSION}")
}
