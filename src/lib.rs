/// Public API surface for integration tests.
/// The binary entry point (main.rs) uses these modules directly;
/// this lib root re-exports them so that files in tests/ can reach them.

pub mod cli;
pub mod metrics;
pub mod platform;
pub mod serve;
pub mod tui;

// Re-export the top-level CLI struct so tests can use `mtop::Cli` directly.
pub use cli::Cli;
