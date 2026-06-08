pub mod c_types;
pub mod c_parser;
pub mod type_map;
pub mod stub_gen;
pub mod cbindgen;
pub mod safety;
pub mod memory;

pub use c_types::{CType, CFunction, CStruct, CEnum, CParam, CField};
pub use type_map::c_type_to_mamba;
pub use safety::{SafeWrapper, ResultConvention};
pub use memory::MemoryBridge;

#[cfg(test)]
#[path = "tests"]
mod tests_files {
    mod abi_semver_gate;
    mod core;
    mod typed_generator_ffi_gate;
}
