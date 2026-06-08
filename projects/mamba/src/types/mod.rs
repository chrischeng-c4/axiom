pub mod ty;
pub mod context;
pub mod check;
pub mod check_stmt;
pub mod check_expr;
pub mod builtins;
pub mod generic;
pub mod protocol;
pub mod stdlib_sigs;
pub mod stdlib_sigs_generated;

pub use ty::{Ty, TypeId};
pub use context::TypeContext;
pub use check::TypeChecker;

#[cfg(test)]
mod tests {
    mod check;
}
