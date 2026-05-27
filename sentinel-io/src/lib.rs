// Export only some functions from scheduler module
mod scheduler;
pub use scheduler::runtime::spawn;
pub use scheduler::runtime::block_on;
pub use scheduler::join::JoinHandle;
pub use scheduler::runtime;

// Time module
pub mod time;

// Macros
pub use sentinel_io_macros::main;

