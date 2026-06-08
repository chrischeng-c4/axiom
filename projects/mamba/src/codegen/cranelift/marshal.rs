use crate::mir::MirType;
use cranelift_codegen::ir::{types as cl_types, InstBuilder, MemFlags, Value};
use cranelift_frontend::FunctionBuilder;

/// Map MirType (C ABI type) to Cranelift IR type (#263).
pub fn mir_type_to_cl(ty: &MirType) -> cranelift_codegen::ir::Type {
    match ty {
        MirType::I8 => cl_types::I8,
        MirType::I32 => cl_types::I32,
        MirType::I64 => cl_types::I64,
        MirType::F32 => cl_types::F32,
        MirType::F64 => cl_types::F64,
        MirType::Ptr => cl_types::I64,
        MirType::Void => cl_types::I64,
    }
}

/// Marshal a Mamba value to a C ABI parameter type (#263).
///
/// Handles narrowing (i64→i32, i64→i8), float demotion (f64→f32),
/// and bool truncation (i64→i8).
pub fn marshal_arg(
    builder: &mut FunctionBuilder,
    value: Value,
    from_cl_type: cranelift_codegen::ir::Type,
    to_mir_type: &MirType,
) -> Value {
    let to_cl = mir_type_to_cl(to_mir_type);
    if from_cl_type == to_cl {
        return value;
    }
    match (from_cl_type, to_mir_type) {
        // Int narrowing: i64 → i32
        (t, MirType::I32) if t == cl_types::I64 => {
            builder.ins().ireduce(cl_types::I32, value)
        }
        // Bool truncation: mamba bool (i64) → C bool (i8)
        (t, MirType::I8) if t == cl_types::I64 => {
            builder.ins().ireduce(cl_types::I8, value)
        }
        // Float demotion: f64 → f32
        (t, MirType::F32) if t == cl_types::F64 => {
            builder.ins().fdemote(cl_types::F32, value)
        }
        // I64→F64 bitcast: all VRegs are I64 (NaN-boxed), extern wants raw F64.
        // Used when calling mb_box_float which expects F64 param.
        (t, MirType::F64) if t == cl_types::I64 => {
            builder.ins().bitcast(cl_types::F64, MemFlags::new(), value)
        }
        // Float→I64 bitcast: NaN-boxed float (raw IEEE 754 bits reinterpreted as u64).
        // Used when passing an F64 variable to a runtime function expecting MbValue (I64).
        (t, MirType::I64) if t == cl_types::F64 => {
            builder.ins().bitcast(cl_types::I64, MemFlags::new(), value)
        }
        // Pointer: pass through as i64
        (_, MirType::Ptr) => value,
        _ => value,
    }
}

/// Unmarshal a C return value back to a Mamba value (#264).
///
/// Handles widening (i32→i64, i8→i64), float promotion (f32→f64),
/// and pointer passthrough.
pub fn unmarshal_return(
    builder: &mut FunctionBuilder,
    value: Value,
    from_mir_type: &MirType,
    to_cl_type: cranelift_codegen::ir::Type,
) -> Value {
    let from_cl = mir_type_to_cl(from_mir_type);
    if from_cl == to_cl_type {
        return value;
    }
    match (from_mir_type, to_cl_type) {
        // Int widening: i32 → i64 (sign-extend)
        (MirType::I32, t) if t == cl_types::I64 => {
            builder.ins().sextend(cl_types::I64, value)
        }
        // Bool widening: i8 → i64 (zero-extend)
        (MirType::I8, t) if t == cl_types::I64 => {
            builder.ins().uextend(cl_types::I64, value)
        }
        // Float promotion: f32 → f64
        (MirType::F32, t) if t == cl_types::F64 => {
            builder.ins().fpromote(cl_types::F64, value)
        }
        // I64→Float bitcast: runtime returns MbValue (I64), dest variable is F64.
        // Reinterpret the NaN-boxed bits back to IEEE 754 f64.
        (MirType::I64, t) if t == cl_types::F64 => {
            builder.ins().bitcast(cl_types::F64, MemFlags::new(), value)
        }
        // F64→I64 bitcast: runtime returns raw F64 (e.g. mb_unbox_float), dest
        // variable is I64 (jit's mamba_to_cl_type always returns I64). Store
        // the raw IEEE 754 bits as i64; subsequent reads bitcast back via
        // VarAlloc::use_as when consumed as float. Without this case the
        // (F64, I64) pair fell through to the wildcard, returning a raw F64
        // value that def_var rejected against an I64-declared variable —
        // surfacing as "declared type of var<N> doesn't match value v<N>" on
        // decorated functions with float-typed parameters.
        (MirType::F64, t) if t == cl_types::I64 => {
            builder.ins().bitcast(cl_types::I64, MemFlags::new(), value)
        }
        // Pointer passthrough
        (MirType::Ptr, _) => value,
        _ => value,
    }
}

/// Determine the Mamba-side Cranelift type for a MirType.
/// Used to know what type we're converting from/to during marshaling.
pub fn mamba_repr_type(mir_ty: &MirType) -> cranelift_codegen::ir::Type {
    match mir_ty {
        MirType::I8 | MirType::I32 | MirType::I64 | MirType::Ptr => cl_types::I64,
        MirType::F32 | MirType::F64 => cl_types::F64,
        MirType::Void => cl_types::I64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mir_type_to_cl() {
        assert_eq!(mir_type_to_cl(&MirType::I8), cl_types::I8);
        assert_eq!(mir_type_to_cl(&MirType::I32), cl_types::I32);
        assert_eq!(mir_type_to_cl(&MirType::I64), cl_types::I64);
        assert_eq!(mir_type_to_cl(&MirType::F32), cl_types::F32);
        assert_eq!(mir_type_to_cl(&MirType::F64), cl_types::F64);
        assert_eq!(mir_type_to_cl(&MirType::Ptr), cl_types::I64);
    }

    #[test]
    fn test_mamba_repr_type() {
        // All integer types are represented as i64 in Mamba
        assert_eq!(mamba_repr_type(&MirType::I8), cl_types::I64);
        assert_eq!(mamba_repr_type(&MirType::I32), cl_types::I64);
        assert_eq!(mamba_repr_type(&MirType::I64), cl_types::I64);
        // All float types are represented as f64 in Mamba
        assert_eq!(mamba_repr_type(&MirType::F32), cl_types::F64);
        assert_eq!(mamba_repr_type(&MirType::F64), cl_types::F64);
    }

    #[test]
    fn test_mir_type_to_cl_void() {
        // Void maps to I64 (no void in Cranelift IR)
        assert_eq!(mir_type_to_cl(&MirType::Void), cl_types::I64);
    }

    #[test]
    fn test_mamba_repr_type_ptr() {
        assert_eq!(mamba_repr_type(&MirType::Ptr), cl_types::I64);
    }

    #[test]
    fn test_mamba_repr_type_void() {
        assert_eq!(mamba_repr_type(&MirType::Void), cl_types::I64);
    }

    #[test]
    fn test_mir_type_to_cl_complete() {
        // Test all MirType variants map to correct Cranelift types
        assert_eq!(mir_type_to_cl(&MirType::I8), cl_types::I8);
        assert_eq!(mir_type_to_cl(&MirType::I32), cl_types::I32);
        assert_eq!(mir_type_to_cl(&MirType::I64), cl_types::I64);
        assert_eq!(mir_type_to_cl(&MirType::F32), cl_types::F32);
        assert_eq!(mir_type_to_cl(&MirType::F64), cl_types::F64);
        assert_eq!(mir_type_to_cl(&MirType::Ptr), cl_types::I64);
        assert_eq!(mir_type_to_cl(&MirType::Void), cl_types::I64);
    }
}
