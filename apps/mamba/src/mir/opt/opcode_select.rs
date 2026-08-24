use crate::mir::*;
use crate::types::TypeContext;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveKind {
    Int(VReg),
    Float(VReg),
    Bool(VReg),
}

pub fn run_opcode_select(body: &mut MirBody, tcx: &TypeContext) -> bool {
    let mut changed = false;
    let mut vreg_kinds: HashMap<VReg, PrimitiveKind> = HashMap::new();
    let mut const_ints: HashMap<VReg, i64> = HashMap::new();
    let mut max_vreg_id: u32 = 0;

    for block in &body.blocks {
        for inst in &block.stmts {
            match inst {
                MirInst::LoadConst { dest, value, .. } => {
                    max_vreg_id = max_vreg_id.max(dest.0);
                    match value {
                        MirConst::Int(val) => {
                            vreg_kinds.insert(*dest, PrimitiveKind::Int(*dest));
                            const_ints.insert(*dest, *val);
                        }
                        MirConst::Float(_) => {
                            vreg_kinds.insert(*dest, PrimitiveKind::Float(*dest));
                        }
                        MirConst::Bool(_) => {
                            vreg_kinds.insert(*dest, PrimitiveKind::Bool(*dest));
                        }
                        _ => {}
                    }
                }
                MirInst::CheckedAdd { dest, .. }
                | MirInst::CheckedSub { dest, .. }
                | MirInst::CheckedMul { dest, .. } => {
                    max_vreg_id = max_vreg_id.max(dest.0);
                    vreg_kinds.insert(*dest, PrimitiveKind::Int(*dest));
                }
                MirInst::BinOp { dest, op, ty, .. } => {
                    max_vreg_id = max_vreg_id.max(dest.0);
                    if *ty == tcx.int() || matches!(op, MirBinOp::Add | MirBinOp::Sub | MirBinOp::Mul | MirBinOp::FloorDiv | MirBinOp::Mod | MirBinOp::BitAnd | MirBinOp::BitOr | MirBinOp::BitXor | MirBinOp::LShift | MirBinOp::RShift) {
                        vreg_kinds.insert(*dest, PrimitiveKind::Int(*dest));
                    } else if *ty == tcx.float() || matches!(op, MirBinOp::Div) {
                        vreg_kinds.insert(*dest, PrimitiveKind::Float(*dest));
                    } else if *ty == tcx.bool() || matches!(op, MirBinOp::Eq | MirBinOp::NotEq | MirBinOp::Lt | MirBinOp::Gt | MirBinOp::LtEq | MirBinOp::GtEq) {
                        vreg_kinds.insert(*dest, PrimitiveKind::Bool(*dest));
                    }
                }
                MirInst::UnaryOp { dest, op, .. } => {
                    max_vreg_id = max_vreg_id.max(dest.0);
                    match op {
                        MirUnaryOp::Neg | MirUnaryOp::Pos | MirUnaryOp::BitNot => {
                            vreg_kinds.insert(*dest, PrimitiveKind::Int(*dest));
                        }
                        MirUnaryOp::Not => {
                            vreg_kinds.insert(*dest, PrimitiveKind::Bool(*dest));
                        }
                    }
                }
                MirInst::Copy { dest, source } => {
                    max_vreg_id = max_vreg_id.max(dest.0).max(source.0);
                    if let Some(&k) = vreg_kinds.get(source) {
                        vreg_kinds.insert(*dest, k);
                    }
                    if let Some(&val) = const_ints.get(source) {
                        const_ints.insert(*dest, val);
                    }
                }
                MirInst::CallExtern { dest, name, args, .. } => {
                    if let Some(d) = dest {
                        max_vreg_id = max_vreg_id.max(d.0);
                    }
                    for arg in args {
                        max_vreg_id = max_vreg_id.max(arg.0);
                    }
                    if name == "mb_box_int" && args.len() == 1 {
                        if let Some(d) = dest {
                            vreg_kinds.insert(*d, PrimitiveKind::Int(args[0]));
                        }
                    } else if name == "mb_box_float" && args.len() == 1 {
                        if let Some(d) = dest {
                            vreg_kinds.insert(*d, PrimitiveKind::Float(args[0]));
                        }
                    } else if name == "mb_box_bool" && args.len() == 1 {
                        if let Some(d) = dest {
                            vreg_kinds.insert(*d, PrimitiveKind::Bool(args[0]));
                        }
                    } else if name == "mb_unbox_int_if_boxed" && args.len() == 1 {
                        if let Some(d) = dest {
                            vreg_kinds.insert(*d, PrimitiveKind::Int(*d));
                        }
                    } else if name == "mb_unbox_bool_if_boxed" && args.len() == 1 {
                        if let Some(d) = dest {
                            vreg_kinds.insert(*d, PrimitiveKind::Bool(*d));
                        }
                    }
                }
                inst => {
                    if let Some(d) = get_inst_dest(inst) {
                        max_vreg_id = max_vreg_id.max(d.0);
                    }
                }
            }
        }
    }

    let mut next_vreg = max_vreg_id + 1;

    for block in &mut body.blocks {
        let mut new_stmts = Vec::with_capacity(block.stmts.len());
        for inst in block.stmts.drain(..) {
            match inst {
                MirInst::CallExtern {
                    dest: Some(dest),
                    ref name,
                    ref args,
                    ty,
                } if name == "mb_dispatch_binop" && args.len() == 3 => {
                    let opcode_vreg = args[0];
                    let boxed_l = args[1];
                    let boxed_r = args[2];

                    if let Some(&op_val) = const_ints.get(&opcode_vreg) {
                        if let Some(binop) = opcode_to_binop(op_val) {
                            let l_kind = vreg_kinds.get(&boxed_l).copied();
                            let r_kind = vreg_kinds.get(&boxed_r).copied();

                            match (l_kind, r_kind) {
                                (Some(PrimitiveKind::Int(raw_l)), Some(PrimitiveKind::Int(raw_r))) => {
                                    let replacement = match binop {
                                        MirBinOp::Add => MirInst::CheckedAdd {
                                            dest,
                                            lhs: raw_l,
                                            rhs: raw_r,
                                            ty: tcx.int(),
                                        },
                                        MirBinOp::Sub => MirInst::CheckedSub {
                                            dest,
                                            lhs: raw_l,
                                            rhs: raw_r,
                                            ty: tcx.int(),
                                        },
                                        MirBinOp::Mul => MirInst::CheckedMul {
                                            dest,
                                            lhs: raw_l,
                                            rhs: raw_r,
                                            ty: tcx.int(),
                                        },
                                        MirBinOp::Div => MirInst::BinOp {
                                            dest,
                                            op: MirBinOp::Div,
                                            lhs: raw_l,
                                            rhs: raw_r,
                                            ty: tcx.float(),
                                        },
                                        _ => {
                                            let res_ty = if is_cmp_op(binop) {
                                                tcx.bool()
                                            } else {
                                                tcx.int()
                                            };
                                            MirInst::BinOp {
                                                dest,
                                                op: binop,
                                                lhs: raw_l,
                                                rhs: raw_r,
                                                ty: res_ty,
                                            }
                                        }
                                    };
                                    vreg_kinds.insert(dest, PrimitiveKind::Int(dest));
                                    new_stmts.push(replacement);
                                    changed = true;
                                    continue;
                                }
                                (Some(PrimitiveKind::Float(raw_l)), Some(PrimitiveKind::Float(raw_r))) => {
                                    let res_ty = if is_cmp_op(binop) {
                                        tcx.bool()
                                    } else {
                                        tcx.float()
                                    };
                                    vreg_kinds.insert(dest, PrimitiveKind::Float(dest));
                                    new_stmts.push(MirInst::BinOp {
                                        dest,
                                        op: binop,
                                        lhs: raw_l,
                                        rhs: raw_r,
                                        ty: res_ty,
                                    });
                                    changed = true;
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                    new_stmts.push(MirInst::CallExtern {
                        dest: Some(dest),
                        name: name.clone(),
                        args: args.clone(),
                        ty,
                    });
                }
                MirInst::CheckedMul { dest, lhs, rhs, ty } => {
                    if let Some((var_vreg, shift_amount)) = try_strength_reduce_mul(lhs, rhs, &const_ints) {
                        let shift_vreg = VReg(next_vreg);
                        next_vreg += 1;
                        new_stmts.push(MirInst::LoadConst {
                            dest: shift_vreg,
                            value: MirConst::Int(shift_amount),
                            ty: tcx.int(),
                        });
                        new_stmts.push(MirInst::BinOp {
                            dest,
                            op: MirBinOp::LShift,
                            lhs: var_vreg,
                            rhs: shift_vreg,
                            ty,
                        });
                        vreg_kinds.insert(dest, PrimitiveKind::Int(dest));
                        changed = true;
                        continue;
                    }
                    new_stmts.push(MirInst::CheckedMul { dest, lhs, rhs, ty });
                }
                MirInst::BinOp { dest, op: MirBinOp::Mul, lhs, rhs, ty } => {
                    if let Some((var_vreg, shift_amount)) = try_strength_reduce_mul(lhs, rhs, &const_ints) {
                        let shift_vreg = VReg(next_vreg);
                        next_vreg += 1;
                        new_stmts.push(MirInst::LoadConst {
                            dest: shift_vreg,
                            value: MirConst::Int(shift_amount),
                            ty: tcx.int(),
                        });
                        new_stmts.push(MirInst::BinOp {
                            dest,
                            op: MirBinOp::LShift,
                            lhs: var_vreg,
                            rhs: shift_vreg,
                            ty,
                        });
                        vreg_kinds.insert(dest, PrimitiveKind::Int(dest));
                        changed = true;
                        continue;
                    }
                    new_stmts.push(MirInst::BinOp { dest, op: MirBinOp::Mul, lhs, rhs, ty });
                }
                other => new_stmts.push(other),
            }
        }
        block.stmts = new_stmts;
    }

    changed
}

const MIN_INT48: i64 = -(1i64 << 47);
const MAX_INT48: i64 = (1i64 << 47) - 1;

fn try_strength_reduce_mul(
    lhs: VReg,
    rhs: VReg,
    const_ints: &HashMap<VReg, i64>,
) -> Option<(VReg, i64)> {
    let is_in_48bit_bounds = |var_vreg: VReg, const_val: i64, shift: i64| -> bool {
        if const_val < MIN_INT48 || const_val > MAX_INT48 {
            return false;
        }
        if let Some(&var_val) = const_ints.get(&var_vreg) {
            if let Some(res) = var_val.checked_shl(shift as u32) {
                return res >= MIN_INT48 && res <= MAX_INT48;
            }
            return false;
        }
        true
    };

    if let Some(&r_val) = const_ints.get(&rhs) {
        if r_val > 1 && (r_val & (r_val - 1)) == 0 {
            let shift = r_val.trailing_zeros() as i64;
            if shift < 64 && is_in_48bit_bounds(lhs, r_val, shift) {
                return Some((lhs, shift));
            }
        }
    }
    if let Some(&l_val) = const_ints.get(&lhs) {
        if l_val > 1 && (l_val & (l_val - 1)) == 0 {
            let shift = l_val.trailing_zeros() as i64;
            if shift < 64 && is_in_48bit_bounds(rhs, l_val, shift) {
                return Some((rhs, shift));
            }
        }
    }
    None
}

fn is_cmp_op(op: MirBinOp) -> bool {
    matches!(
        op,
        MirBinOp::Eq
            | MirBinOp::NotEq
            | MirBinOp::Lt
            | MirBinOp::Gt
            | MirBinOp::LtEq
            | MirBinOp::GtEq
            | MirBinOp::Is
            | MirBinOp::IsNot
            | MirBinOp::In
            | MirBinOp::NotIn
    )
}

fn opcode_to_binop(op_code: i64) -> Option<MirBinOp> {
    match op_code {
        0 => Some(MirBinOp::Add),
        1 => Some(MirBinOp::Sub),
        2 => Some(MirBinOp::Mul),
        3 => Some(MirBinOp::Div),
        4 => Some(MirBinOp::FloorDiv),
        5 => Some(MirBinOp::Mod),
        6 => Some(MirBinOp::Pow),
        7 => Some(MirBinOp::Eq),
        8 => Some(MirBinOp::NotEq),
        9 => Some(MirBinOp::Lt),
        10 => Some(MirBinOp::Gt),
        11 => Some(MirBinOp::LtEq),
        12 => Some(MirBinOp::GtEq),
        13 => Some(MirBinOp::And),
        14 => Some(MirBinOp::Or),
        15 => Some(MirBinOp::BitAnd),
        16 => Some(MirBinOp::BitOr),
        17 => Some(MirBinOp::BitXor),
        18 => Some(MirBinOp::LShift),
        19 => Some(MirBinOp::RShift),
        20 => Some(MirBinOp::Is),
        21 => Some(MirBinOp::IsNot),
        22 => Some(MirBinOp::In),
        23 => Some(MirBinOp::NotIn),
        _ => None,
    }
}

fn get_inst_dest(inst: &MirInst) -> Option<VReg> {
    match inst {
        MirInst::BinOp { dest, .. }
        | MirInst::CheckedAdd { dest, .. }
        | MirInst::CheckedSub { dest, .. }
        | MirInst::CheckedMul { dest, .. }
        | MirInst::UnaryOp { dest, .. }
        | MirInst::LoadConst { dest, .. }
        | MirInst::Copy { dest, .. }
        | MirInst::GetAttr { dest, .. }
        | MirInst::GetItem { dest, .. }
        | MirInst::MakeList { dest, .. }
        | MirInst::MakeDict { dest, .. }
        | MirInst::MakeTuple { dest, .. }
        | MirInst::LoadGlobal { dest, .. }
        | MirInst::LoadCell { dest, .. }
        | MirInst::MakeCell { dest, .. }
        | MirInst::LoadCapture { dest, .. } => Some(*dest),
        MirInst::Call { dest, .. } | MirInst::CallExtern { dest, .. } => *dest,
        _ => None,
    }
}
