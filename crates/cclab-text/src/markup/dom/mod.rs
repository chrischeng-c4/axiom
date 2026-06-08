//! DOM (Document Object Model) types and operations.

mod document;
mod node;

pub use document::Document;
pub use node::{Node, NodeId, NodeType};
