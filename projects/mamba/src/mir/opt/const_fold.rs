use crate::mir::*;
use crate::types::{TypeContext, TypeId};
use std::collections::HashMap;

const MIN_INT48: i64 = -(1i64 << 47);
const MAX_INT48: i64 = (1i64 << 47) - 1;

#[inline]
fn is_int48(v: i64) -> bool {
    v >= MIN_INT48 && v <= MAX_INT48
}

#[inline]
fn is_negative_zero(f: f64) -> bool {
    f == 0.0 && f.to_bits() == (-0.0f64).to_bits()
}

#[inline]
fn py_div_floor(a: i64, b: i64) -> i64 {
    let q = a / b;
    let r = a % b;
    if r != 0 && ((a < 0) ^ (b < 0)) {
        q - 1
    } else {
        q
    }
}

#[inline]
fn py_mod_floor(a: i64, b: i64) -> i64 {
    let r = a % b;
    if r != 0 && ((a < 0) ^ (b < 0)) {
        r + b
    } else {
        r
    }
}

pub fn run_const_fold(body: &mut MirBody, tcx: &TypeContext) -> bool {
    let mut changed = false;
    let mut const_env: HashMap<VReg, (MirConst, TypeId)> = HashMap::new();

    for block in &mut body.blocks {
        let mut new_stmts = Vec::with_capacity(block.stmts.len());
        for inst in block.stmts.drain(..) {
            match inst {
                MirInst::LoadConst { dest, ref value, ty } => {
                    const_env.insert(dest, (value.clone(), ty));
                    new_stmts.push(inst);
                }
                MirInst::Copy { dest, source } => {
                    if let Some((val, ty)) = const_env.get(&source).cloned() {
                        const_env.insert(dest, (val.clone(), ty));
                        new_stmts.push(MirInst::LoadConst {
                            dest,
                            value: val,
                            ty,
                        });
                        changed = true;
                    } else {
                        new_stmts.push(inst);
                    }
                }
                MirInst::BinOp { dest, op, lhs, rhs, ty } => {
                    if let (Some((l_val, _)), Some((r_val, _))) =
                        (const_env.get(&lhs), const_env.get(&rhs))
                    {
                        if let Some((folded_val, folded_ty)) = fold_binop(op, l_val, r_val, ty, tcx) {
                            const_env.insert(dest, (folded_val.clone(), folded_ty));
                            new_stmts.push(MirInst::LoadConst {
                                dest,
                                value: folded_val,
                                ty: folded_ty,
                            });
                            changed = true;
                            continue;
                        }
                    }

                    if let Some((r_val, _)) = const_env.get(&rhs) {
                        match (op, r_val) {
                            (MirBinOp::Add, MirConst::Int(0))
                            | (MirBinOp::Sub, MirConst::Int(0))
                            | (MirBinOp::Mul, MirConst::Int(1))
                            | (MirBinOp::Pow, MirConst::Int(1)) => {
                                if let Some((l_val, l_ty)) = const_env.get(&lhs).cloned() {
                                    const_env.insert(dest, (l_val, l_ty));
                                }
                                new_stmts.push(MirInst::Copy { dest, source: lhs });
                                changed = true;
                                continue;
                            }
                            _ => {}
                        }
                    }
                    if let Some((l_val, _)) = const_env.get(&lhs) {
                        match (op, l_val) {
                            (MirBinOp::Add, MirConst::Int(0))
                            | (MirBinOp::Mul, MirConst::Int(1)) => {
                                if let Some((r_val, r_ty)) = const_env.get(&rhs).cloned() {
                                    const_env.insert(dest, (r_val, r_ty));
                                }
                                new_stmts.push(MirInst::Copy { dest, source: rhs });
                                changed = true;
                                continue;
                            }
                            _ => {}
                        }
                    }

                    new_stmts.push(MirInst::BinOp { dest, op, lhs, rhs, ty });
                }
                MirInst::CheckedAdd { dest, lhs, rhs, ty } => {
                    if let (Some((MirConst::Int(l), _)), Some((MirConst::Int(r), _))) =
                        (const_env.get(&lhs), const_env.get(&rhs))
                    {
                        if let Some(sum) = l.checked_add(*r) {
                            if is_int48(sum) {
                                let val = MirConst::Int(sum);
                                const_env.insert(dest, (val.clone(), ty));
                                new_stmts.push(MirInst::LoadConst { dest, value: val, ty });
                                changed = true;
                                continue;
                            }
                        }
                    }
                    if let Some((MirConst::Int(0), _)) = const_env.get(&rhs) {
                        if let Some((l_val, l_ty)) = const_env.get(&lhs).cloned() {
                            const_env.insert(dest, (l_val, l_ty));
                        }
                        new_stmts.push(MirInst::Copy { dest, source: lhs });
                        changed = true;
                        continue;
                    }
                    if let Some((MirConst::Int(0), _)) = const_env.get(&lhs) {
                        if let Some((r_val, r_ty)) = const_env.get(&rhs).cloned() {
                            const_env.insert(dest, (r_val, r_ty));
                        }
                        new_stmts.push(MirInst::Copy { dest, source: rhs });
                        changed = true;
                        continue;
                    }
                    new_stmts.push(MirInst::CheckedAdd { dest, lhs, rhs, ty });
                }
                MirInst::CheckedSub { dest, lhs, rhs, ty } => {
                    if let (Some((MirConst::Int(l), _)), Some((MirConst::Int(r), _))) =
                        (const_env.get(&lhs), const_env.get(&rhs))
                    {
                        if let Some(diff) = l.checked_sub(*r) {
                            if is_int48(diff) {
                                let val = MirConst::Int(diff);
                                const_env.insert(dest, (val.clone(), ty));
                                new_stmts.push(MirInst::LoadConst { dest, value: val, ty });
                                changed = true;
                                continue;
                            }
                        }
                    }
                    if let Some((MirConst::Int(0), _)) = const_env.get(&rhs) {
                        if let Some((l_val, l_ty)) = const_env.get(&lhs).cloned() {
                            const_env.insert(dest, (l_val, l_ty));
                        }
                        new_stmts.push(MirInst::Copy { dest, source: lhs });
                        changed = true;
                        continue;
                    }
                    new_stmts.push(MirInst::CheckedSub { dest, lhs, rhs, ty });
                }
                MirInst::CheckedMul { dest, lhs, rhs, ty } => {
                    if let (Some((MirConst::Int(l), _)), Some((MirConst::Int(r), _))) =
                        (const_env.get(&lhs), const_env.get(&rhs))
                    {
                        if let Some(prod) = l.checked_mul(*r) {
                            if is_int48(prod) {
                                let val = MirConst::Int(prod);
                                const_env.insert(dest, (val.clone(), ty));
                                new_stmts.push(MirInst::LoadConst { dest, value: val, ty });
                                changed = true;
                                continue;
                            }
                        }
                    }
                    if let Some((MirConst::Int(1), _)) = const_env.get(&rhs) {
                        if let Some((l_val, l_ty)) = const_env.get(&lhs).cloned() {
                            const_env.insert(dest, (l_val, l_ty));
                        }
                        new_stmts.push(MirInst::Copy { dest, source: lhs });
                        changed = true;
                        continue;
                    }
                    if let Some((MirConst::Int(1), _)) = const_env.get(&lhs) {
                        if let Some((r_val, r_ty)) = const_env.get(&rhs).cloned() {
                            const_env.insert(dest, (r_val, r_ty));
                        }
                        new_stmts.push(MirInst::Copy { dest, source: rhs });
                        changed = true;
                        continue;
                    }
                    new_stmts.push(MirInst::CheckedMul { dest, lhs, rhs, ty });
                }
                MirInst::UnaryOp { dest, op, operand, ty } => {
                    if let Some((val, _)) = const_env.get(&operand) {
                        if let Some((folded_val, folded_ty)) = fold_unaryop(op, val, ty, tcx) {
                            const_env.insert(dest, (folded_val.clone(), folded_ty));
                            new_stmts.push(MirInst::LoadConst {
                                dest,
                                value: folded_val,
                                ty: folded_ty,
                            });
                            changed = true;
                            continue;
                        }
                    }
                    new_stmts.push(MirInst::UnaryOp { dest, op, operand, ty });
                }
                MirInst::CallExtern {
                    dest,
                    ref name,
                    ref args,
                    ty,
                } => {
                    if name == "mb_is_truthy" && args.len() == 1 {
                        if let Some(d) = dest {
                            if let Some((val, _)) = const_env.get(&args[0]) {
                                if let Some(b) = eval_truthy(val) {
                                    let folded_val = MirConst::Bool(b);
                                    let folded_ty = tcx.bool();
                                    const_env.insert(d, (folded_val.clone(), folded_ty));
                                    new_stmts.push(MirInst::LoadConst {
                                        dest: d,
                                        value: folded_val,
                                        ty: folded_ty,
                                    });
                                    changed = true;
                                    continue;
                                }
                            }
                        }
                    } else if (name == "mb_unbox_int_if_boxed" || name == "mb_unbox_bool_if_boxed") && args.len() == 1 {
                        if let Some(d) = dest {
                            if let Some((val, vty)) = const_env.get(&args[0]).cloned() {
                                const_env.insert(d, (val.clone(), vty));
                                new_stmts.push(MirInst::LoadConst {
                                    dest: d,
                                    value: val,
                                    ty: vty,
                                });
                                changed = true;
                                continue;
                            }
                        }
                    }
                    new_stmts.push(MirInst::CallExtern {
                        dest,
                        name: name.clone(),
                        args: args.clone(),
                        ty,
                    });
                }
                other => new_stmts.push(other),
            }
        }
        block.stmts = new_stmts;

        if let Terminator::Branch { cond, then_block, else_block } = block.terminator {
            if let Some((val, _)) = const_env.get(&cond) {
                if let Some(b) = eval_truthy(val) {
                    let target = if b { then_block } else { else_block };
                    block.terminator = Terminator::Goto(target);
                    changed = true;
                }
            }
        }
    }

    changed
}

fn fold_binop(
    op: MirBinOp,
    l: &MirConst,
    r: &MirConst,
    _ty: TypeId,
    tcx: &TypeContext,
) -> Option<(MirConst, TypeId)> {
    match (l, r) {
        (MirConst::Int(l_val), MirConst::Int(r_val)) => fold_int_binop(op, *l_val, *r_val, tcx),
        (MirConst::Float(l_val), MirConst::Float(r_val)) => fold_float_binop(op, *l_val, *r_val, tcx),
        (MirConst::Str(l_val), MirConst::Str(r_val)) => fold_str_binop(op, l_val, r_val, tcx),
        (MirConst::Bool(l_val), MirConst::Bool(r_val)) => fold_bool_binop(op, *l_val, *r_val, tcx),
        _ => None,
    }
}

fn fold_int_binop(op: MirBinOp, l: i64, r: i64, tcx: &TypeContext) -> Option<(MirConst, TypeId)> {
    match op {
        MirBinOp::Add => {
            let res = l.checked_add(r)?;
            if is_int48(res) { Some((MirConst::Int(res), tcx.int())) } else { None }
        }
        MirBinOp::Sub => {
            let res = l.checked_sub(r)?;
            if is_int48(res) { Some((MirConst::Int(res), tcx.int())) } else { None }
        }
        MirBinOp::Mul => {
            let res = l.checked_mul(r)?;
            if is_int48(res) { Some((MirConst::Int(res), tcx.int())) } else { None }
        }
        MirBinOp::Div => {
            if r == 0 { return None; }
            let res = (l as f64) / (r as f64);
            if res.is_nan() || res.is_infinite() || is_negative_zero(res) {
                None
            } else {
                Some((MirConst::Float(res), tcx.float()))
            }
        }
        MirBinOp::FloorDiv => {
            if r == 0 { return None; }
            if l == i64::MIN && r == -1 { return None; }
            let res = py_div_floor(l, r);
            if is_int48(res) { Some((MirConst::Int(res), tcx.int())) } else { None }
        }
        MirBinOp::Mod => {
            if r == 0 { return None; }
            if l == i64::MIN && r == -1 { return None; }
            let res = py_mod_floor(l, r);
            if is_int48(res) { Some((MirConst::Int(res), tcx.int())) } else { None }
        }
        MirBinOp::Pow => {
            if r < 0 || r > 32 { return None; }
            let res = l.checked_pow(r as u32)?;
            if is_int48(res) { Some((MirConst::Int(res), tcx.int())) } else { None }
        }
        MirBinOp::BitAnd => Some((MirConst::Int(l & r), tcx.int())),
        MirBinOp::BitOr => Some((MirConst::Int(l | r), tcx.int())),
        MirBinOp::BitXor => Some((MirConst::Int(l ^ r), tcx.int())),
        MirBinOp::LShift => {
            if r < 0 || r >= 64 { return None; }
            let res = l.checked_shl(r as u32)?;
            if is_int48(res) { Some((MirConst::Int(res), tcx.int())) } else { None }
        }
        MirBinOp::RShift => {
            if r < 0 || r >= 64 { return None; }
            let res = l.checked_shr(r as u32)?;
            if is_int48(res) { Some((MirConst::Int(res), tcx.int())) } else { None }
        }
        MirBinOp::Eq => Some((MirConst::Bool(l == r), tcx.bool())),
        MirBinOp::NotEq => Some((MirConst::Bool(l != r), tcx.bool())),
        MirBinOp::Lt => Some((MirConst::Bool(l < r), tcx.bool())),
        MirBinOp::Gt => Some((MirConst::Bool(l > r), tcx.bool())),
        MirBinOp::LtEq => Some((MirConst::Bool(l <= r), tcx.bool())),
        MirBinOp::GtEq => Some((MirConst::Bool(l >= r), tcx.bool())),
        _ => None,
    }
}

fn fold_float_binop(op: MirBinOp, l: f64, r: f64, tcx: &TypeContext) -> Option<(MirConst, TypeId)> {
    if l.is_nan() || r.is_nan() {
        return None;
    }
    match op {
        MirBinOp::Add => {
            let res = l + r;
            if res.is_nan() || res.is_infinite() || is_negative_zero(res) { None } else { Some((MirConst::Float(res), tcx.float())) }
        }
        MirBinOp::Sub => {
            let res = l - r;
            if res.is_nan() || res.is_infinite() || is_negative_zero(res) { None } else { Some((MirConst::Float(res), tcx.float())) }
        }
        MirBinOp::Mul => {
            let res = l * r;
            if res.is_nan() || res.is_infinite() || is_negative_zero(res) { None } else { Some((MirConst::Float(res), tcx.float())) }
        }
        MirBinOp::Div => {
            if r == 0.0 { return None; }
            let res = l / r;
            if res.is_nan() || res.is_infinite() || is_negative_zero(res) { None } else { Some((MirConst::Float(res), tcx.float())) }
        }
        MirBinOp::Eq => Some((MirConst::Bool(l == r), tcx.bool())),
        MirBinOp::NotEq => Some((MirConst::Bool(l != r), tcx.bool())),
        MirBinOp::Lt => Some((MirConst::Bool(l < r), tcx.bool())),
        MirBinOp::Gt => Some((MirConst::Bool(l > r), tcx.bool())),
        MirBinOp::LtEq => Some((MirConst::Bool(l <= r), tcx.bool())),
        MirBinOp::GtEq => Some((MirConst::Bool(l >= r), tcx.bool())),
        _ => None,
    }
}

const MAX_STR_CONST_FOLD_BYTES: usize = 65_536;

fn fold_str_binop(op: MirBinOp, l: &str, r: &str, tcx: &TypeContext) -> Option<(MirConst, TypeId)> {
    match op {
        MirBinOp::Add => {
            if l.len().saturating_add(r.len()) > MAX_STR_CONST_FOLD_BYTES {
                return None;
            }
            Some((MirConst::Str(format!("{l}{r}")), tcx.str()))
        }
        MirBinOp::Eq => Some((MirConst::Bool(l == r), tcx.bool())),
        MirBinOp::NotEq => Some((MirConst::Bool(l != r), tcx.bool())),
        _ => None,
    }
}

fn fold_bool_binop(op: MirBinOp, l: bool, r: bool, tcx: &TypeContext) -> Option<(MirConst, TypeId)> {
    match op {
        MirBinOp::And => Some((MirConst::Bool(l && r), tcx.bool())),
        MirBinOp::Or => Some((MirConst::Bool(l || r), tcx.bool())),
        MirBinOp::Eq => Some((MirConst::Bool(l == r), tcx.bool())),
        MirBinOp::NotEq => Some((MirConst::Bool(l != r), tcx.bool())),
        _ => None,
    }
}

fn fold_unaryop(
    op: MirUnaryOp,
    val: &MirConst,
    _ty: TypeId,
    tcx: &TypeContext,
) -> Option<(MirConst, TypeId)> {
    match (op, val) {
        (MirUnaryOp::Neg, MirConst::Int(v)) => {
            let res = v.checked_neg()?;
            if is_int48(res) {
                Some((MirConst::Int(res), tcx.int()))
            } else {
                None
            }
        }
        (MirUnaryOp::Neg, MirConst::Float(v)) => {
            if v.is_nan() || *v == 0.0 {
                None
            } else {
                let res = -v;
                if is_negative_zero(res) {
                    None
                } else {
                    Some((MirConst::Float(res), tcx.float()))
                }
            }
        }
        (MirUnaryOp::Pos, MirConst::Int(v)) => Some((MirConst::Int(*v), tcx.int())),
        (MirUnaryOp::Pos, MirConst::Float(v)) => {
            if v.is_nan() {
                None
            } else {
                Some((MirConst::Float(*v), tcx.float()))
            }
        }
        (MirUnaryOp::Not, MirConst::Bool(b)) => Some((MirConst::Bool(!b), tcx.bool())),
        (MirUnaryOp::Not, MirConst::Int(v)) => Some((MirConst::Bool(*v == 0), tcx.bool())),
        (MirUnaryOp::Not, MirConst::Str(s)) => Some((MirConst::Bool(s.is_empty()), tcx.bool())),
        (MirUnaryOp::BitNot, MirConst::Int(v)) => Some((MirConst::Int(!v), tcx.int())),
        _ => None,
    }
}

fn eval_truthy(val: &MirConst) -> Option<bool> {
    match val {
        MirConst::Bool(b) => Some(*b),
        MirConst::Int(i) => Some(*i != 0),
        MirConst::Float(f) => {
            if f.is_nan() { None } else { Some(*f != 0.0) }
        }
        MirConst::Str(s) => Some(!s.is_empty()),
        MirConst::None => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_int48_boundaries() {
        assert!(is_int48(0));
        assert!(is_int48(MIN_INT48));
        assert!(is_int48(MAX_INT48));
        assert!(!is_int48(MIN_INT48 - 1));
        assert!(!is_int48(MAX_INT48 + 1));
    }

    #[test]
    fn test_py_div_mod_floor() {
        // Positive / Positive
        assert_eq!(py_div_floor(7, 3), 2);
        assert_eq!(py_mod_floor(7, 3), 1);

        // Negative dividend: -7 // 3 == -3, -7 % 3 == 2
        assert_eq!(py_div_floor(-7, 3), -3);
        assert_eq!(py_mod_floor(-7, 3), 2);

        // Negative divisor: 7 // -3 == -3, 7 % -3 == -2
        assert_eq!(py_div_floor(7, -3), -3);
        assert_eq!(py_mod_floor(7, -3), -2);

        // Negative / Negative: -7 // -3 == 2, -7 % -3 == -1
        assert_eq!(py_div_floor(-7, -3), 2);
        assert_eq!(py_mod_floor(-7, -3), -1);
    }

    #[test]
    fn test_eval_truthy() {
        assert_eq!(eval_truthy(&MirConst::Bool(true)), Some(true));
        assert_eq!(eval_truthy(&MirConst::Bool(false)), Some(false));
        assert_eq!(eval_truthy(&MirConst::Int(10)), Some(true));
        assert_eq!(eval_truthy(&MirConst::Int(0)), Some(false));
        assert_eq!(eval_truthy(&MirConst::Str("hello".to_string())), Some(true));
        assert_eq!(eval_truthy(&MirConst::Str("".to_string())), Some(false));
        assert_eq!(eval_truthy(&MirConst::None), Some(false));
        assert_eq!(eval_truthy(&MirConst::Float(f64::NAN)), None);
    }

    #[test]
    fn test_fold_binop_and_unary() {
        let tcx = TypeContext::new();

        // 10 + 20 -> 30
        let res = fold_binop(MirBinOp::Add, &MirConst::Int(10), &MirConst::Int(20), tcx.int(), &tcx);
        assert_eq!(res.unwrap(), (MirConst::Int(30), tcx.int()));

        // 15 - 5 -> 10
        let res = fold_binop(MirBinOp::Sub, &MirConst::Int(15), &MirConst::Int(5), tcx.int(), &tcx);
        assert_eq!(res.unwrap(), (MirConst::Int(10), tcx.int()));

        // 4 * 5 -> 20
        let res = fold_binop(MirBinOp::Mul, &MirConst::Int(4), &MirConst::Int(5), tcx.int(), &tcx);
        assert_eq!(res.unwrap(), (MirConst::Int(20), tcx.int()));

        // Bitwise AND: 6 & 3 -> 2
        let res = fold_binop(MirBinOp::BitAnd, &MirConst::Int(6), &MirConst::Int(3), tcx.int(), &tcx);
        assert_eq!(res.unwrap(), (MirConst::Int(2), tcx.int()));

        // Unary Negation: -(-42)
        let res = fold_unaryop(MirUnaryOp::Neg, &MirConst::Int(42), tcx.int(), &tcx);
        assert_eq!(res.unwrap(), (MirConst::Int(-42), tcx.int()));

        // Unary Not: not True -> False
        let res = fold_unaryop(MirUnaryOp::Not, &MirConst::Bool(true), tcx.bool(), &tcx);
        assert_eq!(res.unwrap(), (MirConst::Bool(false), tcx.bool()));
    }


    #[test]
    fn test_run_const_fold_mir_body() {
        use crate::resolve::SymbolId;

        let tcx = TypeContext::new();
        let v1 = VReg(1);
        let v2 = VReg(2);
        let v3 = VReg(3);

        let mut body = MirBody {
            name: SymbolId(1),
            params: vec![],
            return_ty: tcx.int(),
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: v1,
                        value: MirConst::Int(10),
                        ty: tcx.int(),
                    },
                    MirInst::LoadConst {
                        dest: v2,
                        value: MirConst::Int(20),
                        ty: tcx.int(),
                    },
                    MirInst::BinOp {
                        dest: v3,
                        op: MirBinOp::Add,
                        lhs: v1,
                        rhs: v2,
                        ty: tcx.int(),
                    },
                ],
                terminator: Terminator::Return(Some(v3)),
            }],
        };

        let changed = run_const_fold(&mut body, &tcx);
        assert!(changed);

        // Third statement should now be LoadConst 30
        if let MirInst::LoadConst { value, .. } = &body.blocks[0].stmts[2] {
            assert_eq!(*value, MirConst::Int(30));
        } else {
            panic!("Expected folded LoadConst instruction for v3");
        }
    }
}


