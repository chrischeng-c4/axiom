use super::super::value::MbValue;

/// __import__(name) — public hook into the import machinery
/// (#1256 sub-priority 2). Honors only `name`; the optional
/// globals/locals/fromlist/level args are dropped at the
/// lower-pass level since Mamba's import path doesn't yet
/// thread package context through. Returns the same module
/// namespace `mb_import` returns for an `import name` stmt.
pub fn mb_dunder_import(name: MbValue) -> MbValue {
    super::super::module::mb_import(name)
}
