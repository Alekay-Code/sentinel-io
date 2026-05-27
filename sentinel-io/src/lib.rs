// Global context
pub(crate) mod context;

// Export only some functions from scheduler module
pub(crate) mod runtime;
pub use runtime::runtime::Runtime;
// pub use runtime::runtime::spawn;
// pub use runtime::runtime::block_on;
pub use runtime::task::join::JoinHandle;

// Time module
pub mod time;

// Macros
pub use sentinel_io_macros::main;
