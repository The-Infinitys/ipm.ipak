// モジュールの宣言
pub mod error;
pub mod graph;
pub mod utils;

#[cfg(test)]
mod tests;

// モジュールから必要な型や関数を再公開
pub use error::{InstallError, RemoveError};
pub use graph::DependencyGraph;
pub use utils::{are_depend_cmds_available, get_missing_depend_cmds};
