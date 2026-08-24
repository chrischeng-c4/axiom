pub mod const_fold;
pub mod opcode_select;
pub mod pass_options;
pub mod peephole;

pub use pass_options::PassOptions;

use crate::driver::config::OptLevel;
use crate::mir::{MirBody, MirModule};
use crate::types::TypeContext;

/// Entry point to optimize all functions in a MIR module with default options.
pub fn optimize_module(module: &mut MirModule, tcx: &TypeContext) {
    let options = PassOptions::from_env();
    optimize_module_with_options(module, tcx, &options);
}

/// Optimize all functions in a MIR module using provided PassOptions.
pub fn optimize_module_with_options(
    module: &mut MirModule,
    tcx: &TypeContext,
    options: &PassOptions,
) {
    for body in &mut module.bodies {
        optimize_body_with_options(body, tcx, options);
    }
}

/// Optimize a single MIR body using iterative optimizer passes until fixed point with default options.
pub fn optimize_body(body: &mut MirBody, tcx: &TypeContext) {
    let options = PassOptions::from_env();
    optimize_body_with_options(body, tcx, &options);
}

/// Optimize a single MIR body using iterative optimizer passes until fixed point with provided PassOptions.
pub fn optimize_body_with_options(
    body: &mut MirBody,
    tcx: &TypeContext,
    options: &PassOptions,
) {
    if options.opt_level == OptLevel::O0
        && !options.enable_const_fold
        && !options.enable_opcode_select
        && !options.enable_peephole
    {
        return;
    }

    let max_iterations = 10;
    for _ in 0..max_iterations {
        let mut changed = false;
        if options.is_pass_enabled("const_fold") {
            changed |= const_fold::run_const_fold(body, tcx);
        }
        if options.is_pass_enabled("opcode_select") {
            changed |= opcode_select::run_opcode_select(body, tcx);
        }
        if options.is_pass_enabled("peephole") {
            changed |= peephole::run_peephole(body, tcx);
        }
        if !changed {
            break;
        }
    }
}

/// Alias for `optimize_module`.
pub fn optimize_mir_module(module: &mut MirModule, tcx: &TypeContext) {
    optimize_module(module, tcx);
}

/// Alias for `optimize_module_with_options`.
pub fn optimize_mir_module_with_options(
    module: &mut MirModule,
    tcx: &TypeContext,
    options: &PassOptions,
) {
    optimize_module_with_options(module, tcx, options);
}

/// Alias for `optimize_body`.
pub fn optimize_mir_body(body: &mut MirBody, tcx: &TypeContext) {
    optimize_body(body, tcx);
}

/// Alias for `optimize_mir_body_with_options`.
pub fn optimize_mir_body_with_options(
    body: &mut MirBody,
    tcx: &TypeContext,
    options: &PassOptions,
) {
    optimize_body_with_options(body, tcx, options);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::*;
    use crate::resolve::SymbolId;

    #[test]
    fn test_optimize_constant_folding() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();

        let mut body = MirBody {
            name: SymbolId(0),
            params: vec![],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Int(10),
                        ty: int_ty,
                    },
                    MirInst::LoadConst {
                        dest: VReg(2),
                        value: MirConst::Int(20),
                        ty: int_ty,
                    },
                    MirInst::CheckedAdd {
                        dest: VReg(3),
                        lhs: VReg(1),
                        rhs: VReg(2),
                        ty: int_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(3))),
            }],
        };

        optimize_body(&mut body, &tcx);

        assert_eq!(body.blocks.len(), 1);
        let block = &body.blocks[0];
        assert_eq!(block.stmts.len(), 1);
        if let MirInst::LoadConst { dest, value, .. } = &block.stmts[0] {
            assert_eq!(*dest, VReg(3));
            assert!(matches!(value, MirConst::Int(30)));
        } else {
            panic!("expected LoadConst dest VReg(3) with Int(30)");
        }
    }

    #[test]
    fn test_optimize_strength_reduction() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();

        let mut body = MirBody {
            name: SymbolId(0),
            params: vec![(VReg(0), int_ty)],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Int(8),
                        ty: int_ty,
                    },
                    MirInst::CheckedMul {
                        dest: VReg(2),
                        lhs: VReg(0),
                        rhs: VReg(1),
                        ty: int_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(2))),
            }],
        };

        optimize_body(&mut body, &tcx);

        let block = &body.blocks[0];
        assert!(block.stmts.iter().any(|inst| matches!(
            inst,
            MirInst::BinOp {
                op: MirBinOp::LShift,
                ..
            }
        )));
    }

    #[test]
    fn test_optimize_jump_folding_and_dce() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();

        let mut body = MirBody {
            name: SymbolId(0),
            params: vec![],
            return_ty: int_ty,
            blocks: vec![
                BasicBlock {
                    id: BlockId(0),
                    stmts: vec![],
                    terminator: Terminator::Goto(BlockId(1)),
                },
                BasicBlock {
                    id: BlockId(1),
                    stmts: vec![],
                    terminator: Terminator::Goto(BlockId(2)),
                },
                BasicBlock {
                    id: BlockId(2),
                    stmts: vec![],
                    terminator: Terminator::Return(None),
                },
                BasicBlock {
                    id: BlockId(3),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(99),
                        value: MirConst::Int(99),
                        ty: int_ty,
                    }],
                    terminator: Terminator::Return(None),
                },
            ],
        };

        optimize_body(&mut body, &tcx);

        if let Terminator::Goto(target) = body.blocks[0].terminator {
            assert_eq!(target, BlockId(2));
        } else {
            panic!("expected Goto(BlockId(2))");
        }
        assert!(body.blocks.iter().all(|b| b.id != BlockId(3)));
    }

    #[test]
    fn test_stress_dos_exponent_caps() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();

        // 2 ** 1000: exponent > 32
        let mut body = MirBody {
            name: SymbolId(10),
            params: vec![],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Int(2),
                        ty: int_ty,
                    },
                    MirInst::LoadConst {
                        dest: VReg(2),
                        value: MirConst::Int(1000),
                        ty: int_ty,
                    },
                    MirInst::BinOp {
                        dest: VReg(3),
                        op: MirBinOp::Pow,
                        lhs: VReg(1),
                        rhs: VReg(2),
                        ty: int_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(3))),
            }],
        };

        optimize_body(&mut body, &tcx);

        // Should NOT fold 2 ** 1000, Pow BinOp must remain
        assert!(body.blocks[0].stmts.iter().any(|inst| matches!(
            inst,
            MirInst::BinOp {
                op: MirBinOp::Pow,
                ..
            }
        )));
    }

    #[test]
    fn test_stress_div_by_zero_safety() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let float_ty = tcx.float();

        // Integer division by zero: 1 / 0
        let mut body = MirBody {
            name: SymbolId(11),
            params: vec![],
            return_ty: float_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Int(1),
                        ty: int_ty,
                    },
                    MirInst::LoadConst {
                        dest: VReg(2),
                        value: MirConst::Int(0),
                        ty: int_ty,
                    },
                    MirInst::BinOp {
                        dest: VReg(3),
                        op: MirBinOp::Div,
                        lhs: VReg(1),
                        rhs: VReg(2),
                        ty: float_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(3))),
            }],
        };

        optimize_body(&mut body, &tcx);

        // BinOp Div must NOT be folded
        assert!(body.blocks[0].stmts.iter().any(|inst| matches!(
            inst,
            MirInst::BinOp {
                op: MirBinOp::Div,
                ..
            }
        )));
    }

    #[test]
    fn test_stress_signed_zero_safety() {
        let tcx = TypeContext::new();
        let float_ty = tcx.float();

        // Unary Neg on 0.0 -> -0.0
        let mut body = MirBody {
            name: SymbolId(12),
            params: vec![],
            return_ty: float_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Float(0.0),
                        ty: float_ty,
                    },
                    MirInst::UnaryOp {
                        dest: VReg(2),
                        op: MirUnaryOp::Neg,
                        operand: VReg(1),
                        ty: float_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(2))),
            }],
        };

        optimize_body(&mut body, &tcx);

        // UnaryOp Neg must NOT be folded into positive 0.0 constant
        assert!(body.blocks[0].stmts.iter().any(|inst| matches!(
            inst,
            MirInst::UnaryOp {
                op: MirUnaryOp::Neg,
                ..
            }
        )));
    }

    #[test]
    fn test_stress_nan_comparisons() {
        let tcx = TypeContext::new();
        let float_ty = tcx.float();
        let bool_ty = tcx.bool();

        // f64::NAN == f64::NAN
        let mut body = MirBody {
            name: SymbolId(13),
            params: vec![],
            return_ty: bool_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Float(f64::NAN),
                        ty: float_ty,
                    },
                    MirInst::LoadConst {
                        dest: VReg(2),
                        value: MirConst::Float(f64::NAN),
                        ty: float_ty,
                    },
                    MirInst::BinOp {
                        dest: VReg(3),
                        op: MirBinOp::Eq,
                        lhs: VReg(1),
                        rhs: VReg(2),
                        ty: bool_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(3))),
            }],
        };

        optimize_body(&mut body, &tcx);

        // BinOp Eq must NOT be folded to Bool(true) on NaNs
        assert!(body.blocks[0].stmts.iter().any(|inst| matches!(
            inst,
            MirInst::BinOp {
                op: MirBinOp::Eq,
                ..
            }
        )));
    }

    #[test]
    fn test_stress_dunder_dispatch_wall_safety() {
        let tcx = TypeContext::new();
        let obj_ty = tcx.any();

        // CallExtern mb_dispatch_binop for non-primitive handle-typed objects
        let mut body = MirBody {
            name: SymbolId(14),
            params: vec![(VReg(1), obj_ty), (VReg(2), obj_ty)],
            return_ty: obj_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(3),
                        value: MirConst::Int(0),
                        ty: tcx.int(),
                    },
                    MirInst::CallExtern {
                        dest: Some(VReg(4)),
                        name: "mb_dispatch_binop".to_string(),
                        args: vec![VReg(3), VReg(1), VReg(2)],
                        ty: obj_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(4))),
            }],
        };

        optimize_body(&mut body, &tcx);

        // Must preserve mb_dispatch_binop for dunder dispatch
        assert!(body.blocks[0].stmts.iter().any(|inst| match inst {
            MirInst::CallExtern { name, .. } => name == "mb_dispatch_binop",
            _ => false,
        }));
    }

    #[test]
    fn test_pass_options_disabling_const_fold() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();

        let mut body = MirBody {
            name: SymbolId(0),
            params: vec![],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Int(10),
                        ty: int_ty,
                    },
                    MirInst::LoadConst {
                        dest: VReg(2),
                        value: MirConst::Int(20),
                        ty: int_ty,
                    },
                    MirInst::CheckedAdd {
                        dest: VReg(3),
                        lhs: VReg(1),
                        rhs: VReg(2),
                        ty: int_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(3))),
            }],
        };

        let mut options = PassOptions::default();
        options.enable_const_fold = false;

        optimize_body_with_options(&mut body, &tcx, &options);

        // CheckedAdd should NOT be folded because const_fold is disabled
        assert!(body.blocks[0].stmts.iter().any(|inst| matches!(
            inst,
            MirInst::CheckedAdd { .. }
        )));
    }

    #[test]
    fn test_str_const_fold_byte_limit() {
        let tcx = TypeContext::new();
        let str_ty = tcx.str();

        let big_a = "a".repeat(40_000);
        let big_b = "b".repeat(30_000);

        let mut body = MirBody {
            name: SymbolId(0),
            params: vec![],
            return_ty: str_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Str(big_a),
                        ty: str_ty,
                    },
                    MirInst::LoadConst {
                        dest: VReg(2),
                        value: MirConst::Str(big_b),
                        ty: str_ty,
                    },
                    MirInst::BinOp {
                        dest: VReg(3),
                        op: MirBinOp::Add,
                        lhs: VReg(1),
                        rhs: VReg(2),
                        ty: str_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(3))),
            }],
        };

        optimize_body(&mut body, &tcx);

        // 40_000 + 30_000 > 65_536 -> Should NOT fold, BinOp::Add remains
        assert!(body.blocks[0].stmts.iter().any(|inst| matches!(
            inst,
            MirInst::BinOp { op: MirBinOp::Add, .. }
        )));
    }

    #[test]
    fn test_algebraic_identities() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();

        // x + 0
        let mut body = MirBody {
            name: SymbolId(0),
            params: vec![(VReg(0), int_ty)],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Int(0),
                        ty: int_ty,
                    },
                    MirInst::CheckedAdd {
                        dest: VReg(2),
                        lhs: VReg(0),
                        rhs: VReg(1),
                        ty: int_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(2))),
            }],
        };

        optimize_body(&mut body, &tcx);

        // CheckedAdd should be replaced with Copy { dest: VReg(2), source: VReg(0) }
        assert!(!body.blocks[0].stmts.iter().any(|inst| matches!(
            inst,
            MirInst::CheckedAdd { .. }
        )));
        assert!(body.blocks[0].stmts.iter().any(|inst| matches!(
            inst,
            MirInst::Copy { dest: VReg(2), source: VReg(0) }
        )));
    }

    #[test]
    fn test_python_floor_div_and_mod_semantics() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();

        // -7 // 3 -> -3
        let mut body = MirBody {
            name: SymbolId(0),
            params: vec![],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Int(-7),
                        ty: int_ty,
                    },
                    MirInst::LoadConst {
                        dest: VReg(2),
                        value: MirConst::Int(3),
                        ty: int_ty,
                    },
                    MirInst::BinOp {
                        dest: VReg(3),
                        op: MirBinOp::FloorDiv,
                        lhs: VReg(1),
                        rhs: VReg(2),
                        ty: int_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(3))),
            }],
        };
        optimize_body(&mut body, &tcx);
        assert_eq!(body.blocks[0].stmts.len(), 1);
        if let MirInst::LoadConst { dest, value, .. } = &body.blocks[0].stmts[0] {
            assert_eq!(*dest, VReg(3));
            assert!(matches!(value, MirConst::Int(-3)));
        } else {
            panic!("expected LoadConst Int(-3)");
        }

        // -7 % 3 -> 2
        let mut body2 = MirBody {
            name: SymbolId(0),
            params: vec![],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Int(-7),
                        ty: int_ty,
                    },
                    MirInst::LoadConst {
                        dest: VReg(2),
                        value: MirConst::Int(3),
                        ty: int_ty,
                    },
                    MirInst::BinOp {
                        dest: VReg(3),
                        op: MirBinOp::Mod,
                        lhs: VReg(1),
                        rhs: VReg(2),
                        ty: int_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(3))),
            }],
        };
        optimize_body(&mut body2, &tcx);
        assert_eq!(body2.blocks[0].stmts.len(), 1);
        if let MirInst::LoadConst { dest, value, .. } = &body2.blocks[0].stmts[0] {
            assert_eq!(*dest, VReg(3));
            assert!(matches!(value, MirConst::Int(2)));
        } else {
            panic!("expected LoadConst Int(2)");
        }
    }

    #[test]
    fn test_strength_reduction_48bit_bounds() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();

        // (1 << 46) * 4 -> (1 << 48) > MAX_INT48 (1 << 47 - 1)
        // Should NOT be strength reduced to LShift because result is out of 48-bit bounds
        let big_val = 1i64 << 46;
        let mut body = MirBody {
            name: SymbolId(0),
            params: vec![],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Int(big_val),
                        ty: int_ty,
                    },
                    MirInst::LoadConst {
                        dest: VReg(2),
                        value: MirConst::Int(4),
                        ty: int_ty,
                    },
                    MirInst::CheckedMul {
                        dest: VReg(3),
                        lhs: VReg(1),
                        rhs: VReg(2),
                        ty: int_ty,
                    },
                ],
                terminator: Terminator::Return(Some(VReg(3))),
            }],
        };

        optimize_body(&mut body, &tcx);

        // CheckedMul should NOT be strength reduced to LShift
        assert!(!body.blocks[0].stmts.iter().any(|inst| matches!(
            inst,
            MirInst::BinOp { op: MirBinOp::LShift, .. }
        )));
    }
}

