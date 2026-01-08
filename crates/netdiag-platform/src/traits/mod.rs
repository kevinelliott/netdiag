//! Platform abstraction traits.

mod autofix;
mod capture;
mod network;
mod privilege;
mod system;
mod wifi;

pub use autofix::*;
pub use capture::*;
pub use network::*;
pub use privilege::*;
pub use system::*;
pub use wifi::*;
