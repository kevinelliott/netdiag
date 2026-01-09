//! Diagnostic result types.

mod jitter;
mod path_analysis;
mod ping;
mod speed;
mod traceroute;

pub use jitter::*;
pub use path_analysis::*;
pub use ping::*;
pub use speed::*;
pub use traceroute::*;
