pub mod c_parser;
pub mod c_types;
pub mod cbindgen;
pub mod memory;
pub mod safety;
pub mod stub_gen;
pub mod type_map;

pub use c_types::{CEnum, CField, CFunction, CParam, CStruct, CType};
pub use memory::MemoryBridge;
pub use safety::{ResultConvention, SafeWrapper};
pub use type_map::c_type_to_mamba;

#[cfg(test)]
#[path = "tests"]
mod tests_files {
    mod abi_semver_gate;
    mod core;
    mod typed_generator_ffi_gate;
}
