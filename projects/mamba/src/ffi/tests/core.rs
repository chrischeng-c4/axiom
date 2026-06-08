#![cfg(test)]

/// End-to-end FFI integration tests (#271).
///
/// Tests the full FFI pipeline: C header → parse → type map → stub gen → MIR → codegen.

use crate::ffi::c_parser::parse_c_header;
use crate::ffi::c_types::*;
use crate::ffi::type_map;
use crate::ffi::stub_gen::generate_tpi_stub;
use crate::ffi::safety::{self, SafeWrapper, ResultConvention};
use crate::ffi::memory::{self, MemoryBridge, FfiAllocation};
use crate::mir::*;
use crate::types::context::TypeContext;
use crate::codegen::cranelift::CraneliftBackend;
use crate::codegen::{CodegenBackend, CodegenOutput};

// ── Pipeline: parse → map → stub ──

#[test]
fn test_full_ffi_pipeline_simple() {
    let header_src = r#"
#ifndef _MATH_H
#define _MATH_H

int32_t add(int32_t a, int32_t b);
double sqrt_f(double x);
void print_int(int32_t val);

#endif
"#;
    let header = parse_c_header(header_src);
    assert_eq!(header.functions.len(), 3);

    let mut tcx = TypeContext::new();
    let mapped = type_map::map_header_types(&header, &mut tcx);
    assert_eq!(mapped.functions.len(), 3);

    let stub = generate_tpi_stub(&header, "math");
    assert!(stub.contains("def add(a: int, b: int) -> int: ..."));
    assert!(stub.contains("def sqrt_f(x: float) -> float: ..."));
    assert!(stub.contains("def print_int(val: int) -> None: ..."));
}

#[test]
fn test_full_ffi_pipeline_struct_and_enum() {
    let header_src = r#"
typedef struct {
    int32_t x;
    int32_t y;
} Point;

typedef enum {
    ColorRed = 0,
    ColorGreen = 1,
    ColorBlue = 2,
} Color;

Point point_new(int32_t x, int32_t y);
int32_t color_value(Color c);
"#;
    let header = parse_c_header(header_src);
    assert_eq!(header.structs.len(), 1);
    assert_eq!(header.enums.len(), 1);
    assert_eq!(header.functions.len(), 2);

    let mut tcx = TypeContext::new();
    let mapped = type_map::map_header_types(&header, &mut tcx);
    assert_eq!(mapped.classes.len(), 2);
    assert_eq!(mapped.functions.len(), 2);

    let stub = generate_tpi_stub(&header, "geometry");
    assert!(stub.contains("class Point:"));
    assert!(stub.contains("x: int"));
    assert!(stub.contains("class Color:"));
    assert!(stub.contains("RED: int = 0"));
    assert!(stub.contains("GREEN: int = 1"));
    assert!(stub.contains("def point_new(x: int, y: int) -> Point: ..."));
}

// ── Codegen: MirModule with externs ──

#[test]
fn test_codegen_with_extern() {
    let tcx = TypeContext::new();
    let int_ty = tcx.int();

    let module = MirModule {
        bodies: vec![MirBody {
            name: crate::resolve::SymbolId(0),
            params: vec![(VReg(0), int_ty), (VReg(1), int_ty)],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    // Call extern "ext_add" with two args
                    MirInst::CallExtern {
                        dest: Some(VReg(2)),
                        name: "ext_add".into(),
                        args: vec![VReg(0), VReg(1)],
                        ty: int_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(2))),
            }],
        }],
        externs: vec![MirExtern {
            name: "ext_add".into(),
            params: vec![MirType::I64, MirType::I64],
            return_type: MirType::I64,
            lib_name: "libext".into(),
        }],
    };

    let mut backend = CraneliftBackend::new().unwrap();
    let output = backend.codegen(&module, &tcx).unwrap();
    match output {
        CodegenOutput::ObjectFile(bytes) => {
            assert!(!bytes.is_empty(), "object file should not be empty");
        }
        _ => panic!("expected ObjectFile output"),
    }
}

#[test]
fn test_codegen_empty_module() {
    let tcx = TypeContext::new();
    let module = MirModule::default();

    #[allow(unused_mut)]
    let mut backend = CraneliftBackend::new().unwrap();
    let output = backend.codegen(&module, &tcx).unwrap();
    match output {
        CodegenOutput::ObjectFile(bytes) => {
            assert!(!bytes.is_empty());
        }
        _ => panic!("expected ObjectFile"),
    }
}

#[test]
fn test_codegen_internal_call() {
    let tcx = TypeContext::new();
    let int_ty = tcx.int();

    let module = MirModule {
        bodies: vec![
            // Function 0: identity(x) -> x
            MirBody {
                name: crate::resolve::SymbolId(0),
                params: vec![(VReg(0), int_ty)],
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![],
                    terminator: Terminator::Return(Some(VReg(0))),
                }],
            },
            // Function 1: caller(x) -> identity(x)
            MirBody {
                name: crate::resolve::SymbolId(1),
                params: vec![(VReg(0), int_ty)],
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::Call {
                        dest: Some(VReg(1)),
                        func: crate::resolve::SymbolId(0),
                        args: vec![VReg(0)],
                        ty: int_ty,
                    }],
                    terminator: Terminator::Return(Some(VReg(1))),
                }],
            },
        ],
        externs: vec![],
    };

    let mut backend = CraneliftBackend::new().unwrap();
    let output = backend.codegen(&module, &tcx).unwrap();
    match output {
        CodegenOutput::ObjectFile(bytes) => assert!(!bytes.is_empty()),
        _ => panic!("expected ObjectFile"),
    }
}

// ── Safety wrappers ──

#[test]
fn test_safety_panic_wrapper_generated() {
    let wrapper = SafeWrapper {
        fn_name: "do_math".into(),
        params: vec![("a".into(), "i64".into()), ("b".into(), "i64".into())],
        return_type: "i64".into(),
        result_convention: ResultConvention::ErrorCode,
    };
    let code = safety::generate_panic_wrapper(&wrapper);
    assert!(code.contains("do_math_safe"));
    assert!(code.contains("catch_unwind"));
    assert!(code.contains("AssertUnwindSafe"));
    assert!(code.contains("do_math(a, b)"));
}

#[test]
fn test_safety_result_check_generated() {
    let code = safety::generate_result_check("fetch_value", "i64");
    assert!(code.contains("fetch_value_checked"));
    assert!(code.contains("out: *mut i64"));
    assert!(code.contains("Ok(val)"));
}

// ── Memory bridge ──

#[test]
fn test_memory_bridge_lifecycle() {
    let mut bridge = MemoryBridge::new();
    assert!(bridge.allocations().is_empty());

    bridge.track(FfiAllocation {
        var_name: "buf".into(),
        c_type: CType::Pointer(Box::new(CType::UInt8)),
        free_fn: "free".into(),
    });
    bridge.track(FfiAllocation {
        var_name: "name".into(),
        c_type: CType::MutChar,
        free_fn: "mamba_free_string".into(),
    });

    assert_eq!(bridge.allocations().len(), 2);

    let cleanup = bridge.generate_cleanup();
    assert!(cleanup.contains("free(buf)"));
    assert!(cleanup.contains("mamba_free_string(name)"));

    bridge.clear();
    assert!(bridge.allocations().is_empty());
}

#[test]
fn test_free_fn_selection() {
    assert_eq!(memory::free_fn_for_type(&CType::ConstChar), "mamba_free_string");
    assert_eq!(memory::free_fn_for_type(&CType::MutChar), "mamba_free_string");
    assert_eq!(
        memory::free_fn_for_type(&CType::Pointer(Box::new(CType::Void))),
        "free"
    );
}

// ── Marshal type mapping ──

#[test]
fn test_marshal_mir_type_mapping() {
    use crate::codegen::cranelift::marshal;
    use cranelift_codegen::ir::types as cl_types;

    assert_eq!(marshal::mir_type_to_cl(&MirType::I32), cl_types::I32);
    assert_eq!(marshal::mir_type_to_cl(&MirType::I64), cl_types::I64);
    assert_eq!(marshal::mir_type_to_cl(&MirType::F64), cl_types::F64);
    assert_eq!(marshal::mir_type_to_cl(&MirType::Ptr), cl_types::I64);

    // Mamba repr types: all ints → i64, all floats → f64
    assert_eq!(marshal::mamba_repr_type(&MirType::I8), cl_types::I64);
    assert_eq!(marshal::mamba_repr_type(&MirType::I32), cl_types::I64);
    assert_eq!(marshal::mamba_repr_type(&MirType::F32), cl_types::F64);
}

// ── Codegen with marshaling (extern returning i32 → widened to i64) ──

#[test]
fn test_codegen_extern_with_marshaling() {
    let tcx = TypeContext::new();
    let int_ty = tcx.int();

    let module = MirModule {
        bodies: vec![MirBody {
            name: crate::resolve::SymbolId(0),
            params: vec![(VReg(0), int_ty)],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    // Call extern that returns i32 (needs widening to i64)
                    MirInst::CallExtern {
                        dest: Some(VReg(1)),
                        name: "c_abs".into(),
                        args: vec![VReg(0)],
                        ty: int_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(1))),
            }],
        }],
        externs: vec![MirExtern {
            name: "c_abs".into(),
            params: vec![MirType::I32],
            return_type: MirType::I32,
            lib_name: "libc".into(),
        }],
    };

    let mut backend = CraneliftBackend::new().unwrap();
    let output = backend.codegen(&module, &tcx).unwrap();
    match output {
        CodegenOutput::ObjectFile(bytes) => {
            assert!(!bytes.is_empty(), "should produce object with extern call");
        }
        _ => panic!("expected ObjectFile"),
    }
}
