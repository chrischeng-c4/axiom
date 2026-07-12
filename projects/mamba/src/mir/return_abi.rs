use std::collections::{HashMap, HashSet};

use crate::types::{Ty, TypeContext, TypeId};

use super::{
    DynamicReturnAdapter, MirBinOp, MirBody, MirConst, MirInst, MirType, MirUnaryOp,
    PhysicalReturn, ReturnAbi, ReturnOwnership, Terminator, VReg,
};

pub(crate) type ExternReturnAbi = (MirType, Option<ReturnAbi>);

/// Producer-level lattice used to classify physical ABI across every CFG edge.
/// `BOXED_INT` stays separate from general boxed values so the only mixed
/// primitive contract published by the analysis is the real raw-or-BigInt case.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PhysicalReturnSet(u8);

impl PhysicalReturnSet {
    const RAW_INT: Self = Self(1 << 0);
    const RAW_BOOL: Self = Self(1 << 1);
    const RAW_FLOAT: Self = Self(1 << 2);
    const BOXED_INT: Self = Self(1 << 3);
    const BOXED_VALUE: Self = Self(1 << 4);
    const UNKNOWN: Self = Self(1 << 5);
    const DEFERRED: Self = Self(1 << 6);

    fn insert(&mut self, other: Self) -> bool {
        let old = self.0;
        self.0 |= other.0;
        old != self.0
    }

    fn boxed_for_type(ty: TypeId, tcx: &TypeContext) -> Self {
        if matches!(tcx.get(ty), Ty::Int) {
            Self::BOXED_INT
        } else {
            Self::BOXED_VALUE
        }
    }

    fn from_abi(abi: ReturnAbi, ty: TypeId, tcx: &TypeContext) -> Self {
        match abi.physical {
            PhysicalReturn::RawInt => Self::RAW_INT,
            PhysicalReturn::RawBool => Self::RAW_BOOL,
            PhysicalReturn::RawFloat => Self::RAW_FLOAT,
            PhysicalReturn::BoxedMbValue => Self::boxed_for_type(ty, tcx),
            PhysicalReturn::BoxedOrRawFloat => Self(Self::RAW_FLOAT.0 | Self::BOXED_VALUE.0),
            PhysicalReturn::RawOrBoxedInt => Self(Self::RAW_INT.0 | Self::BOXED_INT.0),
            PhysicalReturn::Unknown => Self::UNKNOWN,
        }
    }

    fn into_abi(self) -> Option<ReturnAbi> {
        let bits = self.0 & !Self::DEFERRED.0;
        if bits == 0 {
            return None;
        }
        let boxed = Self::BOXED_INT.0 | Self::BOXED_VALUE.0;
        let physical = match bits {
            bits if bits & Self::UNKNOWN.0 != 0 => PhysicalReturn::Unknown,
            bits if bits == Self::RAW_INT.0 => PhysicalReturn::RawInt,
            bits if bits == Self::RAW_BOOL.0 => PhysicalReturn::RawBool,
            bits if bits == Self::RAW_FLOAT.0 => PhysicalReturn::RawFloat,
            bits if bits & !boxed == 0 => PhysicalReturn::BoxedMbValue,
            bits if bits & Self::RAW_INT.0 != 0
                && bits & !(Self::RAW_INT.0 | Self::BOXED_INT.0) == 0 =>
            {
                PhysicalReturn::RawOrBoxedInt
            }
            // Internal float bits and boxed values are both complete MbValue
            // bit patterns in the JIT's I64 return register.
            bits if bits & (Self::RAW_INT.0 | Self::RAW_BOOL.0) == 0
                && bits & Self::RAW_FLOAT.0 != 0
                && bits & boxed != 0 =>
            {
                PhysicalReturn::BoxedOrRawFloat
            }
            _ => PhysicalReturn::Unknown,
        };
        let ownership = match physical {
            PhysicalReturn::RawInt | PhysicalReturn::RawBool | PhysicalReturn::RawFloat => {
                ReturnOwnership::NoHeapOwner
            }
            PhysicalReturn::BoxedMbValue => ReturnOwnership::NewlyOwnedBoxed,
            PhysicalReturn::BoxedOrRawFloat
            | PhysicalReturn::RawOrBoxedInt
            | PhysicalReturn::Unknown => ReturnOwnership::ProvenanceTransfer,
        };
        Some(ReturnAbi::new(physical, ownership))
    }
}

/// Canonical producer analysis for one MIR body.
///
/// Lowering consumes `return_abi`; Cranelift consumes the per-VReg projection.
/// Keeping both on one result prevents representation guesses from drifting.
#[derive(Clone)]
pub(crate) struct BodyPhysicalAbiAnalysis {
    values: HashMap<VReg, PhysicalReturnSet>,
    returned: PhysicalReturnSet,
}

#[derive(Clone)]
pub(crate) struct ModulePhysicalAbiAnalysis {
    body_abis: HashMap<u32, ReturnAbi>,
    bodies: HashMap<u32, BodyPhysicalAbiAnalysis>,
}

impl ModulePhysicalAbiAnalysis {
    pub(crate) fn body_return_abi(&self, symbol: u32) -> Option<ReturnAbi> {
        self.body_abis.get(&symbol).copied()
    }

    pub(crate) fn body(&self, symbol: u32) -> Option<&BodyPhysicalAbiAnalysis> {
        self.bodies.get(&symbol)
    }
}

fn legacy_dynamic_return_adapter(body: &MirBody, tcx: &TypeContext) -> DynamicReturnAdapter {
    // Freeze the pre-#1448 dynamic-dispatch decision while the canonical
    // physical ABI replaces its semantic approximation. #1452 owns removing
    // this adapter when dispatch can consume full provenance.
    let mut vreg_ty: HashMap<VReg, TypeId> = HashMap::new();
    let mut copy_src: HashMap<VReg, VReg> = HashMap::new();
    for (vreg, ty) in &body.params {
        vreg_ty.insert(*vreg, *ty);
    }
    for block in &body.blocks {
        for inst in &block.stmts {
            match inst {
                MirInst::BinOp { dest, ty, .. }
                | MirInst::CheckedAdd { dest, ty, .. }
                | MirInst::CheckedSub { dest, ty, .. }
                | MirInst::CheckedMul { dest, ty, .. }
                | MirInst::UnaryOp { dest, ty, .. }
                | MirInst::LoadConst { dest, ty, .. }
                | MirInst::GetAttr { dest, ty, .. }
                | MirInst::GetItem { dest, ty, .. }
                | MirInst::MakeList { dest, ty, .. }
                | MirInst::MakeDict { dest, ty, .. }
                | MirInst::MakeTuple { dest, ty, .. }
                | MirInst::LoadGlobal { dest, ty, .. }
                | MirInst::LoadCell { dest, ty, .. }
                | MirInst::MakeCell { dest, ty, .. }
                | MirInst::LoadCapture { dest, ty, .. } => {
                    vreg_ty.insert(*dest, *ty);
                }
                MirInst::Call {
                    dest: Some(dest),
                    ty,
                    ..
                }
                | MirInst::CallExtern {
                    dest: Some(dest),
                    ty,
                    ..
                } => {
                    vreg_ty.insert(*dest, *ty);
                }
                MirInst::Copy { dest, source } => {
                    copy_src.insert(*dest, *source);
                }
                _ => {}
            }
        }
    }
    let resolve = |mut vreg: VReg| -> Option<TypeId> {
        for _ in 0..64 {
            if let Some(ty) = vreg_ty.get(&vreg) {
                return Some(*ty);
            }
            vreg = *copy_src.get(&vreg)?;
        }
        None
    };

    let mut returns_value = false;
    for block in &body.blocks {
        if let Terminator::Return(Some(vreg)) = &block.terminator {
            returns_value = true;
            if matches!(
                resolve(*vreg).map(|ty| tcx.get(ty)),
                Some(Ty::Int) | Some(Ty::Bool) | None
            ) {
                return DynamicReturnAdapter::BoxRawInt;
            }
        }
    }
    if returns_value {
        DynamicReturnAdapter::BypassIntBoxing
    } else {
        DynamicReturnAdapter::BoxRawInt
    }
}

pub(crate) fn analyze_module_physical_abis(
    bodies: &[MirBody],
    tcx: &TypeContext,
    extern_abis: &HashMap<String, ExternReturnAbi>,
) -> ModulePhysicalAbiAnalysis {
    let body_return_tys: HashMap<u32, TypeId> = bodies
        .iter()
        .map(|body| (body.name.0, body.return_ty))
        .collect();
    let native_bool_bodies: HashSet<u32> = bodies
        .iter()
        .filter(|body| super::body_returns_native_bool(body, tcx))
        .map(|body| body.name.0)
        .collect();
    let mut body_abis = HashMap::new();

    // Calls can target bodies declared later. Iterate over a stable snapshot
    // so forward aliases and mutually-referential bodies converge without
    // using a declared semantic return type as a physical guess.
    for _ in 0..=bodies.len() {
        let mut next = body_abis.clone();
        for body in bodies {
            if let Some(abi) = analyze_body_physical_abis(
                body,
                tcx,
                &body_abis,
                &body_return_tys,
                &native_bool_bodies,
                extern_abis,
            )
            .return_abi()
            {
                next.insert(
                    body.name.0,
                    abi.with_dynamic_adapter(legacy_dynamic_return_adapter(body, tcx)),
                );
            }
        }
        if next == body_abis {
            break;
        }
        body_abis = next;
    }

    let mut body_values = HashMap::new();
    for body in bodies {
        body_values.insert(
            body.name.0,
            analyze_body_physical_abis(
                body,
                tcx,
                &body_abis,
                &body_return_tys,
                &native_bool_bodies,
                extern_abis,
            ),
        );
        body_abis.entry(body.name.0).or_insert_with(|| {
            ReturnAbi::new(PhysicalReturn::Unknown, ReturnOwnership::ProvenanceTransfer)
                .with_dynamic_adapter(legacy_dynamic_return_adapter(body, tcx))
        });
    }

    ModulePhysicalAbiAnalysis {
        body_abis,
        bodies: body_values,
    }
}

impl BodyPhysicalAbiAnalysis {
    pub(crate) fn return_abi(&self) -> Option<ReturnAbi> {
        self.returned.into_abi()
    }

    /// Physical per-VReg projection only. Ownership remains a producer
    /// contract and is wired explicitly by #1450; this analysis must not
    /// reconstruct it from representation bits.
    pub(crate) fn value_physical(&self, vreg: VReg) -> Option<PhysicalReturn> {
        self.values
            .get(&vreg)
            .copied()?
            .into_abi()
            .map(|abi| abi.physical)
    }

    pub(crate) fn raw_or_boxed_int_vregs(&self) -> HashSet<VReg> {
        self.values
            .iter()
            .filter_map(|(vreg, set)| {
                matches!(
                    set.into_abi().map(|abi| abi.physical),
                    Some(PhysicalReturn::RawOrBoxedInt)
                )
                .then_some(*vreg)
            })
            .collect()
    }
}

fn parameter_return_set(ty: TypeId, tcx: &TypeContext) -> PhysicalReturnSet {
    // Primitive entry types carry their current JIT ABI. Nonprimitive params
    // remain unknown until #1451 owns argument adaptation: an Any-typed value
    // can already contain raw bits that static call boxing cannot recognize.
    match tcx.get(ty) {
        // Raw-int entry adaptation preserves an out-of-range boxed BigInt.
        Ty::Int => PhysicalReturnSet(PhysicalReturnSet::RAW_INT.0 | PhysicalReturnSet::BOXED_INT.0),
        Ty::Bool => PhysicalReturnSet::RAW_BOOL,
        Ty::Float => PhysicalReturnSet::RAW_FLOAT,
        _ => PhysicalReturnSet::UNKNOWN,
    }
}

fn analyze_body_physical_abis(
    body: &MirBody,
    tcx: &TypeContext,
    body_abis: &HashMap<u32, ReturnAbi>,
    body_return_tys: &HashMap<u32, TypeId>,
    native_bool_bodies: &HashSet<u32>,
    extern_abis: &HashMap<String, ExternReturnAbi>,
) -> BodyPhysicalAbiAnalysis {
    let mut values: HashMap<VReg, PhysicalReturnSet> = HashMap::new();
    for (vreg, ty) in &body.params {
        values
            .entry(*vreg)
            .or_default()
            .insert(parameter_return_set(*ty, tcx));
    }

    let instruction_count = body
        .blocks
        .iter()
        .map(|block| block.stmts.len())
        .sum::<usize>();
    for _ in 0..=instruction_count {
        let mut changed = false;
        for block in &body.blocks {
            for inst in &block.stmts {
                let produced = match inst {
                    MirInst::LoadConst { dest, value, .. } => Some((
                        *dest,
                        match value {
                            MirConst::Int(_) => PhysicalReturnSet::RAW_INT,
                            MirConst::BigInt(_) => PhysicalReturnSet::BOXED_INT,
                            MirConst::Float(_) => PhysicalReturnSet::RAW_FLOAT,
                            MirConst::Bool(_) => PhysicalReturnSet::RAW_BOOL,
                            _ => PhysicalReturnSet::BOXED_VALUE,
                        },
                    )),
                    MirInst::CheckedAdd { dest, .. }
                    | MirInst::CheckedSub { dest, .. }
                    | MirInst::CheckedMul { dest, .. } => Some((
                        *dest,
                        PhysicalReturnSet(
                            PhysicalReturnSet::RAW_INT.0 | PhysicalReturnSet::BOXED_INT.0,
                        ),
                    )),
                    MirInst::BinOp { dest, op, ty, .. } => {
                        let set = if matches!(op, MirBinOp::In | MirBinOp::NotIn) {
                            PhysicalReturnSet::UNKNOWN
                        } else if matches!(
                            op,
                            MirBinOp::Eq
                                | MirBinOp::NotEq
                                | MirBinOp::Lt
                                | MirBinOp::Gt
                                | MirBinOp::LtEq
                                | MirBinOp::GtEq
                                | MirBinOp::Is
                                | MirBinOp::IsNot
                        ) {
                            PhysicalReturnSet::RAW_BOOL
                        } else {
                            match tcx.get(*ty) {
                                Ty::Int => PhysicalReturnSet(
                                    PhysicalReturnSet::RAW_INT.0 | PhysicalReturnSet::BOXED_INT.0,
                                ),
                                Ty::Bool => PhysicalReturnSet::RAW_BOOL,
                                Ty::Float => PhysicalReturnSet::RAW_FLOAT,
                                _ => PhysicalReturnSet::UNKNOWN,
                            }
                        };
                        Some((*dest, set))
                    }
                    MirInst::UnaryOp { dest, op, ty, .. } => {
                        let set = if matches!(op, MirUnaryOp::Not) {
                            PhysicalReturnSet::RAW_BOOL
                        } else {
                            match tcx.get(*ty) {
                                Ty::Int => PhysicalReturnSet(
                                    PhysicalReturnSet::RAW_INT.0 | PhysicalReturnSet::BOXED_INT.0,
                                ),
                                Ty::Bool => PhysicalReturnSet::RAW_BOOL,
                                Ty::Float => PhysicalReturnSet::RAW_FLOAT,
                                _ => PhysicalReturnSet::UNKNOWN,
                            }
                        };
                        Some((*dest, set))
                    }
                    MirInst::Call {
                        dest: Some(dest),
                        func,
                        ty,
                        ..
                    } => {
                        let abi = body_abis.get(&func.0).copied();
                        let callsite_is_nonprimitive =
                            !matches!(tcx.get(*ty), Ty::Int | Ty::Bool | Ty::Float);
                        let set = if abi.is_none() && body_return_tys.contains_key(&func.0) {
                            PhysicalReturnSet::DEFERRED
                        } else if callsite_is_nonprimitive {
                            match body_return_tys.get(&func.0).map(|ty| tcx.get(*ty)) {
                                Some(Ty::Int) => PhysicalReturnSet::BOXED_INT,
                                Some(Ty::Bool) => PhysicalReturnSet::BOXED_VALUE,
                                Some(Ty::Float) => abi
                                    .map(|abi| PhysicalReturnSet::from_abi(abi, *ty, tcx))
                                    .unwrap_or(PhysicalReturnSet::UNKNOWN),
                                _ if native_bool_bodies.contains(&func.0) => {
                                    PhysicalReturnSet::BOXED_VALUE
                                }
                                _ => abi
                                    .map(|abi| PhysicalReturnSet::from_abi(abi, *ty, tcx))
                                    .unwrap_or(PhysicalReturnSet::UNKNOWN),
                            }
                        } else {
                            abi.map(|abi| PhysicalReturnSet::from_abi(abi, *ty, tcx))
                                .unwrap_or(PhysicalReturnSet::UNKNOWN)
                        };
                        Some((*dest, set))
                    }
                    MirInst::CallExtern {
                        dest: Some(dest),
                        name,
                        ty,
                        ..
                    } => {
                        let set = match extern_abis.get(name) {
                            Some((MirType::Void, _)) => PhysicalReturnSet::BOXED_VALUE,
                            Some((_, Some(abi))) => PhysicalReturnSet::from_abi(*abi, *ty, tcx),
                            _ => PhysicalReturnSet::UNKNOWN,
                        };
                        Some((*dest, set))
                    }
                    MirInst::Copy { dest, source } => {
                        values.get(source).copied().map(|set| (*dest, set))
                    }
                    MirInst::GetAttr { dest, .. }
                    | MirInst::GetItem { dest, .. }
                    | MirInst::LoadGlobal { dest, .. }
                    | MirInst::LoadCell { dest, .. }
                    | MirInst::LoadCapture { dest, .. } => {
                        Some((*dest, PhysicalReturnSet::UNKNOWN))
                    }
                    MirInst::MakeList { dest, .. }
                    | MirInst::MakeDict { dest, .. }
                    | MirInst::MakeTuple { dest, .. }
                    | MirInst::MakeCell { dest, .. } => {
                        Some((*dest, PhysicalReturnSet::BOXED_VALUE))
                    }
                    _ => None,
                };
                if let Some((dest, set)) = produced {
                    changed |= values.entry(dest).or_default().insert(set);
                }
            }
        }
        if !changed {
            break;
        }
    }

    let mut returned = PhysicalReturnSet::default();
    for block in &body.blocks {
        match &block.terminator {
            Terminator::Return(Some(vreg)) => {
                returned.insert(
                    values
                        .get(vreg)
                        .copied()
                        .unwrap_or(PhysicalReturnSet::UNKNOWN),
                );
            }
            Terminator::Return(None) => {
                returned.insert(PhysicalReturnSet::BOXED_VALUE);
            }
            _ => {}
        }
    }
    BodyPhysicalAbiAnalysis { values, returned }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{BasicBlock, BlockId};
    use crate::resolve::SymbolId;

    fn analyze(body: &MirBody, tcx: &TypeContext) -> BodyPhysicalAbiAnalysis {
        analyze_body_physical_abis(
            body,
            tcx,
            &HashMap::new(),
            &HashMap::new(),
            &HashSet::new(),
            &HashMap::new(),
        )
    }

    #[test]
    fn per_vreg_analysis_selects_only_raw_or_boxed_ints() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let body = MirBody {
            name: SymbolId(1),
            params: vec![(VReg(0), int_ty)],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Int(7),
                        ty: int_ty,
                    },
                    MirInst::LoadConst {
                        dest: VReg(2),
                        value: MirConst::BigInt("1208925819614629174706176".into()),
                        ty: int_ty,
                    },
                    MirInst::CheckedAdd {
                        dest: VReg(3),
                        lhs: VReg(0),
                        rhs: VReg(1),
                        ty: int_ty,
                    },
                    MirInst::Copy {
                        dest: VReg(4),
                        source: VReg(3),
                    },
                ],
                terminator: Terminator::Return(Some(VReg(4))),
            }],
        };

        let analysis = analyze(&body, &tcx);
        assert_eq!(
            analysis.raw_or_boxed_int_vregs(),
            HashSet::from([VReg(0), VReg(3), VReg(4)])
        );
        assert_eq!(
            analysis.value_physical(VReg(1)).unwrap(),
            PhysicalReturn::RawInt
        );
        assert_eq!(
            analysis.value_physical(VReg(2)).unwrap(),
            PhysicalReturn::BoxedMbValue
        );
    }

    #[test]
    fn per_vreg_analysis_merges_cfg_producers_without_bits_inference() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let body = MirBody {
            name: SymbolId(2),
            params: vec![],
            return_ty: int_ty,
            blocks: vec![
                BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Bool(true),
                        ty: tcx.bool(),
                    }],
                    terminator: Terminator::Branch {
                        cond: VReg(1),
                        then_block: BlockId(1),
                        else_block: BlockId(2),
                    },
                },
                BasicBlock {
                    id: BlockId(1),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(0),
                        value: MirConst::Int(1),
                        ty: int_ty,
                    }],
                    terminator: Terminator::Goto(BlockId(3)),
                },
                BasicBlock {
                    id: BlockId(2),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(0),
                        value: MirConst::BigInt("1208925819614629174706176".into()),
                        ty: int_ty,
                    }],
                    terminator: Terminator::Goto(BlockId(3)),
                },
                BasicBlock {
                    id: BlockId(3),
                    stmts: vec![],
                    terminator: Terminator::Return(Some(VReg(0))),
                },
            ],
        };

        let analysis = analyze(&body, &tcx);
        assert_eq!(analysis.raw_or_boxed_int_vregs(), HashSet::from([VReg(0)]));
        assert_eq!(
            analysis.return_abi().unwrap().physical,
            PhysicalReturn::RawOrBoxedInt
        );
    }

    #[test]
    fn per_vreg_analysis_consumes_explicit_extern_abi() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let body = MirBody {
            name: SymbolId(3),
            params: vec![],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::CallExtern {
                        dest: Some(VReg(0)),
                        name: "explicit_mixed".into(),
                        args: vec![],
                        ty: int_ty,
                    },
                    MirInst::CallExtern {
                        dest: Some(VReg(1)),
                        name: "explicit_borrowed".into(),
                        args: vec![],
                        ty: tcx.any(),
                    },
                ],
                terminator: Terminator::Return(Some(VReg(0))),
            }],
        };
        let extern_abis = HashMap::from([
            (
                "explicit_mixed".to_owned(),
                (
                    MirType::I64,
                    Some(ReturnAbi::new(
                        PhysicalReturn::RawOrBoxedInt,
                        ReturnOwnership::ProvenanceTransfer,
                    )),
                ),
            ),
            (
                "explicit_borrowed".to_owned(),
                (
                    MirType::I64,
                    Some(ReturnAbi::new(
                        PhysicalReturn::BoxedMbValue,
                        ReturnOwnership::BorrowedBoxed,
                    )),
                ),
            ),
        ]);

        let analysis = analyze_body_physical_abis(
            &body,
            &tcx,
            &HashMap::new(),
            &HashMap::new(),
            &HashSet::new(),
            &extern_abis,
        );
        assert_eq!(analysis.raw_or_boxed_int_vregs(), HashSet::from([VReg(0)]));
        assert_eq!(
            analysis.value_physical(VReg(1)),
            Some(PhysicalReturn::BoxedMbValue)
        );
    }

    #[test]
    fn module_analysis_converges_forward_internal_call_values() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let caller = MirBody {
            name: SymbolId(10),
            params: vec![],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![MirInst::Call {
                    dest: Some(VReg(0)),
                    func: SymbolId(11),
                    args: vec![],
                    ty: int_ty,
                }],
                terminator: Terminator::Return(Some(VReg(0))),
            }],
        };
        let callee = MirBody {
            name: SymbolId(11),
            params: vec![],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![MirInst::LoadConst {
                    dest: VReg(0),
                    value: MirConst::Int(7),
                    ty: int_ty,
                }],
                terminator: Terminator::Return(Some(VReg(0))),
            }],
        };

        let analysis = analyze_module_physical_abis(&[caller, callee], &tcx, &HashMap::new());
        assert_eq!(
            analysis.body_return_abi(10).unwrap().physical,
            PhysicalReturn::RawInt
        );
        assert_eq!(
            analysis.body(10).unwrap().value_physical(VReg(0)),
            Some(PhysicalReturn::RawInt)
        );
    }

    #[test]
    fn module_analysis_converges_recursive_edge_from_concrete_base() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let bool_ty = tcx.bool();
        let symbol = SymbolId(12);
        let body = MirBody {
            name: symbol,
            params: vec![],
            return_ty: int_ty,
            blocks: vec![
                BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(2),
                        value: MirConst::Bool(true),
                        ty: bool_ty,
                    }],
                    terminator: Terminator::Branch {
                        cond: VReg(2),
                        then_block: BlockId(1),
                        else_block: BlockId(2),
                    },
                },
                BasicBlock {
                    id: BlockId(1),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(0),
                        value: MirConst::Int(1),
                        ty: int_ty,
                    }],
                    terminator: Terminator::Return(Some(VReg(0))),
                },
                BasicBlock {
                    id: BlockId(2),
                    stmts: vec![MirInst::Call {
                        dest: Some(VReg(1)),
                        func: symbol,
                        args: vec![],
                        ty: int_ty,
                    }],
                    terminator: Terminator::Return(Some(VReg(1))),
                },
            ],
        };

        let analysis = analyze_module_physical_abis(&[body], &tcx, &HashMap::new());
        assert_eq!(
            analysis.body_return_abi(symbol.0).unwrap().physical,
            PhysicalReturn::RawInt
        );
        assert_eq!(
            analysis.body(symbol.0).unwrap().value_physical(VReg(1)),
            Some(PhysicalReturn::RawInt)
        );
    }

    #[test]
    fn module_analysis_applies_nonprimitive_callsite_boxing() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let any_ty = tcx.any();
        let caller = MirBody {
            name: SymbolId(13),
            params: vec![],
            return_ty: any_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![MirInst::Call {
                    dest: Some(VReg(0)),
                    func: SymbolId(14),
                    args: vec![],
                    ty: any_ty,
                }],
                terminator: Terminator::Return(Some(VReg(0))),
            }],
        };
        let callee = MirBody {
            name: SymbolId(14),
            params: vec![],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![MirInst::LoadConst {
                    dest: VReg(0),
                    value: MirConst::Int(7),
                    ty: int_ty,
                }],
                terminator: Terminator::Return(Some(VReg(0))),
            }],
        };

        let analysis = analyze_module_physical_abis(&[caller, callee], &tcx, &HashMap::new());
        assert_eq!(
            analysis.body_return_abi(13).unwrap().physical,
            PhysicalReturn::BoxedMbValue
        );
        assert_eq!(
            analysis.body(13).unwrap().value_physical(VReg(0)),
            Some(PhysicalReturn::BoxedMbValue)
        );
    }
}
