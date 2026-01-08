//! Report format implementations.

mod html;
mod json;
mod markdown;
mod text;

pub use html::HtmlFormatter;
pub use json::JsonFormatter;
pub use markdown::MarkdownFormatter;
pub use text::TextFormatter;
