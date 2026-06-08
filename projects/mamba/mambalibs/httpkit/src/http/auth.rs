//! Authentication carried on the request side.

#[derive(Debug, Clone, Default)]
pub enum Auth {
    #[default]
    None,
    Basic {
        username: String,
        password: String,
    },
    Bearer(String),
}
