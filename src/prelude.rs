pub mod ipak {
    use std::str::FromStr;

    /// バージョン構造体を利用できるようにしています。
    pub use crate::utils::version::{Version, VersionRange};
    /// ipak自身のバージョンを、Version構造体で返します。
    pub fn version() -> Version {
        Version::from_str(
            option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
        ).expect("Failed to parse CARGO_PKG_VERSION. This is not a normal behavior.")
    }
}
