//! Markup processing module (HTML/XML, CSS selectors, XPath, XSLT).

pub mod css;
pub mod dom;
mod error;
pub mod html;
pub mod xml;
pub mod xpath;
pub mod xslt;

pub use css::select;
pub use dom::{Document, Node, NodeId, NodeType};
pub use error::{MarkupError, Result};
pub use html::parse_html;
pub use xml::parse_xml;
pub use xpath::xpath;
pub use xslt::transform;
