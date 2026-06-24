pub mod builtins;
pub mod check;
pub mod check_expr;
pub mod check_stmt;
pub mod context;
pub mod generic;
pub mod protocol;
pub mod stdlib_sigs;
pub mod stdlib_sigs_generated;
pub mod ty;

pub use check::TypeChecker;
pub use context::TypeContext;
pub use ty::{Ty, TypeId};

#[cfg(test)]
mod tests {
    mod check;
}
