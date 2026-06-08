pub mod cranelift;
pub mod llvm;

use crate::mir::{MirBody, MirModule};
use crate::types::TypeContext;

/// Output produced by a codegen backend.
pub enum CodegenOutput {
    /// Raw object file bytes (ELF, Mach-O, COFF)
    ObjectFile(Vec<u8>),
    /// WASM bytecode
    Wasm(Vec<u8>),
    /// In-memory JIT function pointer
    Jit { entry: *const u8 },
    /// Textual LLVM IR (when llc is not available for object emission)
    LlvmIr(String),
}

/// Trait for pluggable code generation backends.
pub trait CodegenBackend {
    /// Compile a complete MIR module (functions + externs) to native code.
    fn codegen(&mut self, module: &MirModule, tcx: &TypeContext) -> crate::error::Result<CodegenOutput>;
    fn name(&self) -> &str;
}

/// Legacy helper: compile only function bodies (no externs).
pub fn codegen_bodies(
    backend: &mut dyn CodegenBackend,
    bodies: &[MirBody],
    tcx: &TypeContext,
) -> crate::error::Result<CodegenOutput> {
    let module = MirModule {
        bodies: bodies.to_vec(),
        externs: vec![],
    };
    backend.codegen(&module, tcx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{BasicBlock, BlockId, MirConst, MirInst, Terminator, VReg};
    use crate::resolve::SymbolId;

    #[test]
    fn test_codegen_output_variants() {
        // Verify all CodegenOutput variants are constructible
        let obj = CodegenOutput::ObjectFile(vec![0x7f, 0x45, 0x4c, 0x46]);
        matches!(obj, CodegenOutput::ObjectFile(_));

        let wasm = CodegenOutput::Wasm(vec![0x00, 0x61, 0x73, 0x6d]);
        matches!(wasm, CodegenOutput::Wasm(_));

        let jit = CodegenOutput::Jit { entry: std::ptr::null() };
        matches!(jit, CodegenOutput::Jit { .. });

        let ir = CodegenOutput::LlvmIr("define i64 @main()".to_string());
        matches!(ir, CodegenOutput::LlvmIr(_));
    }

    #[test]
    fn test_codegen_bodies_wraps_in_module() {
        // codegen_bodies should wrap bodies into a MirModule with empty externs
        struct DummyBackend;
        impl CodegenBackend for DummyBackend {
            fn codegen(
                &mut self,
                module: &MirModule,
                _tcx: &TypeContext,
            ) -> crate::error::Result<CodegenOutput> {
                assert!(module.externs.is_empty());
                assert_eq!(module.bodies.len(), 1);
                Ok(CodegenOutput::LlvmIr("ok".to_string()))
            }
            fn name(&self) -> &str { "dummy" }
        }

        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let bodies = vec![MirBody {
            name: SymbolId(0),
            params: vec![],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![MirInst::LoadConst {
                    dest: VReg(0),
                    value: MirConst::Int(0),
                    ty: int_ty,
                }],
                terminator: Terminator::Return(Some(VReg(0))),
            }],
        }];

        let mut backend = DummyBackend;
        let result = codegen_bodies(&mut backend, &bodies, &tcx).unwrap();
        assert!(matches!(result, CodegenOutput::LlvmIr(s) if s == "ok"));
    }
}
