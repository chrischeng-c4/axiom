// HANDWRITE-BEGIN gap="missing-generator:mamba-strict-type-provenance-proof-matrix" tracker="#1453" reason="The cross-backend raw-or-boxed Int matrix is an executable regression catalogue, not a runtime code-generation primitive."
//! Executable strict-type provenance matrix (#1453).
//!
//! Existing focused tests own individual lowering contracts.  This module
//! connects their adversarial cases to real JIT execution and an AOT link/run
//! smoke so the proof cannot degrade into an IR-only accounting exercise.

use super::jit::{CraneliftJitBackend, JIT_LOCK};
use super::CraneliftBackend;
use crate::codegen::{CodegenBackend, CodegenOutput};
use crate::mir::{
    BasicBlock, BlockId, MirBinOp, MirBody, MirConst, MirInst, MirModule, MirUnaryOp, Terminator,
    VReg,
};
use crate::resolve::SymbolId;
use crate::runtime::{rc, return_owner, symbols, MbValue};
use crate::types::TypeContext;
use std::process::Command;

fn entry_module(stmts: Vec<MirInst>, returned: VReg, tcx: &TypeContext) -> MirModule {
    MirModule {
        bodies: vec![MirBody {
            name: SymbolId(u32::MAX),
            params: vec![],
            return_ty: tcx.int(),
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts,
                terminator: Terminator::Return(Some(returned)),
            }],
        }],
        externs: vec![],
    }
}

fn call_jit_zero_arg(tcx: &TypeContext, module: &MirModule) -> (CraneliftJitBackend, i64) {
    let _jit_guard = JIT_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut backend = CraneliftJitBackend::new().expect("JIT init");
    backend
        .codegen(module, tcx)
        .expect("JIT provenance codegen");
    let entry = backend
        .get_func_ptr(u32::MAX)
        .expect("JIT provenance entry address");
    let call: extern "C" fn() -> i64 = unsafe { std::mem::transmute(entry) };
    (backend, call())
}

fn raw_constant_case(tcx: &TypeContext, value: i64) -> MirModule {
    entry_module(
        vec![MirInst::LoadConst {
            dest: VReg(0),
            value: MirConst::Int(value),
            ty: tcx.int(),
        }],
        VReg(0),
        tcx,
    )
}

fn copy_and_operator_case(tcx: &TypeContext) -> MirModule {
    let int = tcx.int();
    entry_module(
        vec![
            MirInst::LoadConst {
                dest: VReg(0),
                value: MirConst::Int(10),
                ty: int,
            },
            MirInst::Copy {
                dest: VReg(1),
                source: VReg(0),
            },
            MirInst::LoadConst {
                dest: VReg(2),
                value: MirConst::Int(6),
                ty: int,
            },
            MirInst::BinOp {
                dest: VReg(3),
                op: MirBinOp::BitXor,
                lhs: VReg(1),
                rhs: VReg(2),
                ty: int,
            },
            MirInst::UnaryOp {
                dest: VReg(4),
                op: MirUnaryOp::BitNot,
                operand: VReg(3),
                ty: int,
            },
            MirInst::LoadConst {
                dest: VReg(5),
                value: MirConst::Int(1),
                ty: int,
            },
            MirInst::BinOp {
                dest: VReg(6),
                op: MirBinOp::RShift,
                lhs: VReg(4),
                rhs: VReg(5),
                ty: int,
            },
        ],
        VReg(6),
        tcx,
    )
}

#[test]
fn jit_producer_matrix_preserves_explicit_owners() {
    return_owner::clear_return_owner_frames_for_test();
    let tcx = TypeContext::new();

    let (_raw_backend, raw) = call_jit_zero_arg(&tcx, &raw_constant_case(&tcx, 37));
    assert_eq!(raw, 37);
    assert!(return_owner::take_matching_return_owner(MbValue::from_int(raw)).is_none());

    let (_copy_backend, copied) = call_jit_zero_arg(&tcx, &copy_and_operator_case(&tcx));
    assert_eq!(copied, -7);
    assert!(return_owner::take_matching_return_owner(MbValue::from_int(copied)).is_none());

    // The canonical producer-owner analysis, and both concrete backends, must
    // mention every raw-or-boxed producer class.  This catches a new MIR arm
    // that bypasses the shared companion contract before it gains fixtures.
    let owner_contract = include_str!("../../mir/producer_owner.rs");
    let jit = include_str!("jit.rs");
    let object = include_str!("mod.rs");
    for producer in [
        "LoadConst",
        "LoadGlobal",
        "LoadCell",
        "LoadCapture",
        "GetAttr",
        "GetItem",
        "BinOp",
        "UnaryOp",
        "CheckedAdd",
        "CheckedSub",
        "CheckedMul",
        "Copy",
        "Call",
        "CallExtern",
    ] {
        assert!(
            owner_contract.contains(producer),
            "missing owner contract for {producer}"
        );
        assert!(
            jit.contains(&format!("MirInst::{producer}")),
            "missing JIT route for {producer}"
        );
        assert!(
            object.contains(&format!("MirInst::{producer}")),
            "missing Object route for {producer}"
        );
    }
    assert!(owner_contract.contains("mb_unbox_int_if_boxed"));
}

#[test]
fn pointer_shaped_raw_values_never_adopt_bigint_owner() {
    return_owner::clear_return_owner_frames_for_test();
    let tcx = TypeContext::new();
    let live_bigint = crate::runtime::bigint_ops::bigint_from_i128(1_i128 << 70);
    let pointer_shaped = live_bigint.as_ptr().expect("BigInt").cast::<()>() as usize as i64;
    assert!(
        (-(1_i64 << 47)..(1_i64 << 47)).contains(&pointer_shaped),
        "host pointer payload must fit the JIT raw-int test lane"
    );

    // The payload intentionally has the exact bits of a live allocation.  It
    // is still an ordinary raw i64 producer and cannot acquire that owner.
    let (_raw_backend, raw) = call_jit_zero_arg(&tcx, &raw_constant_case(&tcx, pointer_shaped));
    assert_eq!(raw, pointer_shaped);
    let raw_value = MbValue::from_int(raw);
    assert!(symbols::mb_typed_int_owner_or_none(raw_value).is_none());
    assert_eq!(
        symbols::mb_typed_int_owner_or_none(live_bigint),
        live_bigint
    );
    assert_eq!(
        unsafe { rc::mb_refcount(live_bigint.as_ptr().expect("BigInt")) },
        1
    );

    unsafe { rc::release_if_ptr(live_bigint) };
    assert!(include_str!("jit.rs").contains("companion_owner_raw_collision_and_bigint_refcounts"));
}

#[test]
fn aot_provenance_object_executes_with_host_linker() {
    let tcx = TypeContext::new();
    let module = raw_constant_case(&tcx, 37);
    let bytes = match CraneliftBackend::new()
        .expect("AOT init")
        .codegen(&module, &tcx)
        .expect("AOT provenance codegen")
    {
        CodegenOutput::ObjectFile(bytes) => bytes,
        _ => panic!("AOT backend returned non-object output"),
    };

    let stamp = format!(
        "{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );
    let root = std::env::temp_dir();
    let object = root.join(format!("mamba_provenance_{stamp}.o"));
    let executable = root.join(format!("mamba_provenance_{stamp}"));
    std::fs::write(&object, bytes).expect("write AOT object");

    let link = Command::new("cc")
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("invoke host linker for AOT provenance proof");
    assert!(
        link.status.success(),
        "host linker failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&link.stdout),
        String::from_utf8_lossy(&link.stderr)
    );
    let run = Command::new(&executable)
        .output()
        .expect("execute linked AOT provenance object");
    assert!(
        run.status.success(),
        "AOT provenance executable failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), "37");

    let _ = std::fs::remove_file(object);
    let _ = std::fs::remove_file(executable);
}

// marker: missing-generator:mamba-strict-type-provenance-proof-matrix
// HANDWRITE-END
