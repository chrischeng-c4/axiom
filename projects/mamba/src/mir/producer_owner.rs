use std::collections::{HashMap, HashSet};

use super::{
    BlockId, MirBinOp, MirBody, MirInst, MirType, MirUnaryOp, PhysicalReturn, ReturnAbi,
    ReturnOwnership, VReg,
};

pub(super) type ExternReturnAbi = (MirType, Option<ReturnAbi>);

/// Stable location of a MIR value producer.
///
/// Instruction sites use semantic block ids rather than block-array indexes so
/// metadata remains attached to the producer when blocks are reordered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ProducerSite {
    Parameter {
        index: usize,
    },
    Instruction {
        block: BlockId,
        statement_index: usize,
    },
}

/// Ownership boundaries intentionally left for their dedicated migrations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ProducerBoundary {
    /// Argument representation and ownership adaptation is owned by #1451.
    ParameterIngress,
    /// Direct internal return companions are carried across calls by #1452.
    InternalReturn,
    /// Unknown/dynamic runtime return companions are carried by #1452.
    DynamicReturn,
}

/// Where codegen must obtain the owner value used by an ownership action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum OwnerValueSource {
    /// Copy-like producer: retain the source VReg's companion owner.
    SourceCompanion(VReg),
}

/// Typed companion contract for one extern result.
///
/// These variants are actionable without examining result bits or semantic
/// `TypeId`. #1461 owns implementing the runtime sidecars they describe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ExternCompanionContract {
    OwnerlessOrImmortal,
    FreshResultOrNone,
    ArgumentPassThroughOrNone {
        source: VReg,
    },
    ExplicitOwnerOut,
    DeferredRuntimeReturn,
    MissingDeclaration,
    MissingReturnAbi,
    MissingArgument {
        index: usize,
    },
    Invalid {
        physical: PhysicalReturn,
        ownership: ReturnOwnership,
    },
}

impl ExternCompanionContract {
    fn for_call(name: &str, args: &[VReg], return_type: MirType, abi: Option<ReturnAbi>) -> Self {
        if let Some(declared) = declared_extern_companion(name) {
            if !matches!(
                abi,
                Some(ReturnAbi {
                    physical: PhysicalReturn::RawOrBoxedInt,
                    ownership: ReturnOwnership::ProvenanceTransfer,
                    ..
                })
            ) {
                return abi.map_or(Self::MissingReturnAbi, |abi| Self::Invalid {
                    physical: abi.physical,
                    ownership: abi.ownership,
                });
            }
            return match declared {
                DeclaredExternCompanion::FreshResultOrNone => Self::FreshResultOrNone,
                DeclaredExternCompanion::ArgumentPassThroughOrNone { argument_index } => args
                    .get(argument_index)
                    .copied()
                    .map(|source| Self::ArgumentPassThroughOrNone { source })
                    .unwrap_or(Self::MissingArgument {
                        index: argument_index,
                    }),
            };
        }

        let Some(abi) = abi else {
            return if return_type == MirType::I64 {
                Self::MissingReturnAbi
            } else {
                Self::OwnerlessOrImmortal
            };
        };

        match (abi.physical, abi.ownership) {
            (
                PhysicalReturn::RawInt | PhysicalReturn::RawBool | PhysicalReturn::RawFloat,
                ReturnOwnership::NoHeapOwner,
            ) => Self::OwnerlessOrImmortal,
            (PhysicalReturn::BoxedMbValue, ReturnOwnership::NewlyOwnedBoxed) => {
                Self::FreshResultOrNone
            }
            (PhysicalReturn::Unknown, ReturnOwnership::ProvenanceTransfer) => {
                Self::DeferredRuntimeReturn
            }
            (PhysicalReturn::RawOrBoxedInt, ReturnOwnership::ProvenanceTransfer) => {
                Self::ExplicitOwnerOut
            }
            (physical, ownership) => Self::Invalid {
                physical,
                ownership,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum DeclaredExternCompanion {
    FreshResultOrNone,
    ArgumentPassThroughOrNone { argument_index: usize },
}

/// Temporary MIR-owned declaration table. #1461 projects these contracts from
/// runtime sidecar metadata; keeping the table centralized prevents callsites
/// from rediscovering provenance from payload bits in the meantime.
fn declared_extern_companion(name: &str) -> Option<DeclaredExternCompanion> {
    match name {
        "mb_pow_int" | "mb_bigint_add" | "mb_bigint_sub" | "mb_bigint_mul" => {
            Some(DeclaredExternCompanion::FreshResultOrNone)
        }
        "mb_unbox_int_if_boxed" | "mb_unbox_inline_int_if_boxed" => {
            Some(DeclaredExternCompanion::ArgumentPassThroughOrNone { argument_index: 0 })
        }
        _ => None,
    }
}

/// Owner operation attached to a concrete producer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ProducerOwnerAction {
    OwnerlessOrImmortal,
    FreshResultOrNone,
    PassThroughOrNone(OwnerValueSource),
    ExplicitOwnerOut,
    Extern(ExternCompanionContract),
    DeferredBoundary(ProducerBoundary),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ProducerOwnerMetadata {
    pub(crate) dest: VReg,
    pub(crate) action: ProducerOwnerAction,
}

pub(super) fn analyze_producer_owners(
    body: &MirBody,
    extern_abis: &HashMap<String, ExternReturnAbi>,
    relevant_vregs: &HashSet<VReg>,
) -> HashMap<ProducerSite, ProducerOwnerMetadata> {
    let mut producers = HashMap::new();

    for (index, (dest, _)) in body.params.iter().enumerate() {
        if !relevant_vregs.contains(dest) {
            continue;
        }
        producers.insert(
            ProducerSite::Parameter { index },
            ProducerOwnerMetadata {
                dest: *dest,
                action: ProducerOwnerAction::DeferredBoundary(ProducerBoundary::ParameterIngress),
            },
        );
    }

    for block in &body.blocks {
        for (statement_index, inst) in block.stmts.iter().enumerate() {
            let Some(metadata) = instruction_owner(inst, extern_abis) else {
                continue;
            };
            if !relevant_vregs.contains(&metadata.dest) {
                continue;
            }
            producers.insert(
                ProducerSite::Instruction {
                    block: block.id,
                    statement_index,
                },
                metadata,
            );
        }
    }

    producers
}

fn instruction_owner(
    inst: &MirInst,
    extern_abis: &HashMap<String, ExternReturnAbi>,
) -> Option<ProducerOwnerMetadata> {
    let producer = value_producer(inst)?;
    let dest = producer.dest();
    let action = match producer {
        ValueProducer::BinOp { op, .. } => binop_owner(op),
        ValueProducer::CheckedAdd { .. }
        | ValueProducer::CheckedSub { .. }
        | ValueProducer::CheckedMul { .. } => ProducerOwnerAction::FreshResultOrNone,
        ValueProducer::UnaryOp { op, operand, .. } => unary_owner(op, operand),
        ValueProducer::LoadConst { value, .. } => match value {
            super::MirConst::BigInt(_) | super::MirConst::Str(_) | super::MirConst::Bytes(_) => {
                ProducerOwnerAction::FreshResultOrNone
            }
            super::MirConst::Int(_)
            | super::MirConst::Float(_)
            | super::MirConst::Bool(_)
            | super::MirConst::None
            | super::MirConst::NotImplemented
            | super::MirConst::Ellipsis
            | super::MirConst::FuncRef(_)
            | super::MirConst::ExternFuncRef(_) => ProducerOwnerAction::OwnerlessOrImmortal,
        },
        ValueProducer::Call { .. } => {
            ProducerOwnerAction::DeferredBoundary(ProducerBoundary::InternalReturn)
        }
        ValueProducer::Copy { source, .. } => {
            ProducerOwnerAction::PassThroughOrNone(OwnerValueSource::SourceCompanion(source))
        }
        ValueProducer::CallExtern { name, args, .. } => {
            let contract = extern_abis
                .get(name)
                .map(|(return_type, abi)| {
                    ExternCompanionContract::for_call(name, args, *return_type, *abi)
                })
                .unwrap_or(ExternCompanionContract::MissingDeclaration);
            ProducerOwnerAction::Extern(contract)
        }
        ValueProducer::GetAttr { .. }
        | ValueProducer::GetItem { .. }
        | ValueProducer::LoadGlobal { .. }
        | ValueProducer::LoadCell { .. }
        | ValueProducer::LoadCapture { .. } => {
            ProducerOwnerAction::DeferredBoundary(ProducerBoundary::DynamicReturn)
        }
        ValueProducer::MakeList { .. }
        | ValueProducer::MakeDict { .. }
        | ValueProducer::MakeTuple { .. } => ProducerOwnerAction::FreshResultOrNone,
        // Cell handles are non-owning inline ids; the cell storage owns its
        // contained value independently.
        ValueProducer::MakeCell { .. } => ProducerOwnerAction::OwnerlessOrImmortal,
    };

    Some(ProducerOwnerMetadata { dest, action })
}

fn binop_owner(op: MirBinOp) -> ProducerOwnerAction {
    match op {
        MirBinOp::Add
        | MirBinOp::Sub
        | MirBinOp::Mul
        | MirBinOp::Div
        | MirBinOp::FloorDiv
        | MirBinOp::Mod
        | MirBinOp::Pow
        | MirBinOp::And
        | MirBinOp::Or
        | MirBinOp::BitAnd
        | MirBinOp::BitOr
        | MirBinOp::BitXor
        | MirBinOp::LShift
        | MirBinOp::RShift => ProducerOwnerAction::ExplicitOwnerOut,
        MirBinOp::In | MirBinOp::NotIn => {
            ProducerOwnerAction::DeferredBoundary(ProducerBoundary::DynamicReturn)
        }
        MirBinOp::Eq
        | MirBinOp::NotEq
        | MirBinOp::Lt
        | MirBinOp::Gt
        | MirBinOp::LtEq
        | MirBinOp::GtEq
        | MirBinOp::Is
        | MirBinOp::IsNot => ProducerOwnerAction::OwnerlessOrImmortal,
    }
}

fn unary_owner(op: MirUnaryOp, operand: VReg) -> ProducerOwnerAction {
    match op {
        MirUnaryOp::Pos => {
            ProducerOwnerAction::PassThroughOrNone(OwnerValueSource::SourceCompanion(operand))
        }
        MirUnaryOp::Neg | MirUnaryOp::BitNot => ProducerOwnerAction::ExplicitOwnerOut,
        MirUnaryOp::Not => ProducerOwnerAction::OwnerlessOrImmortal,
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ValueProducerKind {
    BinOp,
    CheckedAdd,
    CheckedSub,
    CheckedMul,
    UnaryOp,
    LoadConst,
    Call,
    Copy,
    CallExtern,
    GetAttr,
    GetItem,
    MakeList,
    MakeDict,
    MakeTuple,
    LoadGlobal,
    LoadCell,
    MakeCell,
    LoadCapture,
}

enum ValueProducer<'a> {
    BinOp {
        dest: VReg,
        op: MirBinOp,
    },
    CheckedAdd {
        dest: VReg,
    },
    CheckedSub {
        dest: VReg,
    },
    CheckedMul {
        dest: VReg,
    },
    UnaryOp {
        dest: VReg,
        op: MirUnaryOp,
        operand: VReg,
    },
    LoadConst {
        dest: VReg,
        value: &'a super::MirConst,
    },
    Call {
        dest: VReg,
    },
    Copy {
        dest: VReg,
        source: VReg,
    },
    CallExtern {
        dest: VReg,
        name: &'a str,
        args: &'a [VReg],
    },
    GetAttr {
        dest: VReg,
    },
    GetItem {
        dest: VReg,
    },
    MakeList {
        dest: VReg,
    },
    MakeDict {
        dest: VReg,
    },
    MakeTuple {
        dest: VReg,
    },
    LoadGlobal {
        dest: VReg,
    },
    LoadCell {
        dest: VReg,
    },
    MakeCell {
        dest: VReg,
    },
    LoadCapture {
        dest: VReg,
    },
}

impl ValueProducer<'_> {
    fn dest(&self) -> VReg {
        match self {
            Self::BinOp { dest, .. }
            | Self::CheckedAdd { dest }
            | Self::CheckedSub { dest }
            | Self::CheckedMul { dest }
            | Self::UnaryOp { dest, .. }
            | Self::LoadConst { dest, .. }
            | Self::Call { dest }
            | Self::Copy { dest, .. }
            | Self::CallExtern { dest, .. }
            | Self::GetAttr { dest }
            | Self::GetItem { dest }
            | Self::MakeList { dest }
            | Self::MakeDict { dest }
            | Self::MakeTuple { dest }
            | Self::LoadGlobal { dest }
            | Self::LoadCell { dest }
            | Self::MakeCell { dest }
            | Self::LoadCapture { dest } => *dest,
        }
    }

    #[cfg(test)]
    fn kind(&self) -> ValueProducerKind {
        match self {
            Self::BinOp { .. } => ValueProducerKind::BinOp,
            Self::CheckedAdd { .. } => ValueProducerKind::CheckedAdd,
            Self::CheckedSub { .. } => ValueProducerKind::CheckedSub,
            Self::CheckedMul { .. } => ValueProducerKind::CheckedMul,
            Self::UnaryOp { .. } => ValueProducerKind::UnaryOp,
            Self::LoadConst { .. } => ValueProducerKind::LoadConst,
            Self::Call { .. } => ValueProducerKind::Call,
            Self::Copy { .. } => ValueProducerKind::Copy,
            Self::CallExtern { .. } => ValueProducerKind::CallExtern,
            Self::GetAttr { .. } => ValueProducerKind::GetAttr,
            Self::GetItem { .. } => ValueProducerKind::GetItem,
            Self::MakeList { .. } => ValueProducerKind::MakeList,
            Self::MakeDict { .. } => ValueProducerKind::MakeDict,
            Self::MakeTuple { .. } => ValueProducerKind::MakeTuple,
            Self::LoadGlobal { .. } => ValueProducerKind::LoadGlobal,
            Self::LoadCell { .. } => ValueProducerKind::LoadCell,
            Self::MakeCell { .. } => ValueProducerKind::MakeCell,
            Self::LoadCapture { .. } => ValueProducerKind::LoadCapture,
        }
    }
}

/// Exhaustive MirInst inventory. Adding an instruction cannot compile until it
/// is explicitly declared value-producing or non-producing here; every value
/// producer then flows through the exhaustive `ValueProducer` action match.
fn value_producer(inst: &MirInst) -> Option<ValueProducer<'_>> {
    match inst {
        MirInst::BinOp { dest, op, .. } => Some(ValueProducer::BinOp {
            dest: *dest,
            op: *op,
        }),
        MirInst::CheckedAdd { dest, .. } => Some(ValueProducer::CheckedAdd { dest: *dest }),
        MirInst::CheckedSub { dest, .. } => Some(ValueProducer::CheckedSub { dest: *dest }),
        MirInst::CheckedMul { dest, .. } => Some(ValueProducer::CheckedMul { dest: *dest }),
        MirInst::UnaryOp {
            dest, op, operand, ..
        } => Some(ValueProducer::UnaryOp {
            dest: *dest,
            op: *op,
            operand: *operand,
        }),
        MirInst::LoadConst { dest, value, .. } => {
            Some(ValueProducer::LoadConst { dest: *dest, value })
        }
        MirInst::Call {
            dest: Some(dest), ..
        } => Some(ValueProducer::Call { dest: *dest }),
        MirInst::Call { dest: None, .. } => None,
        MirInst::Copy { dest, source } => Some(ValueProducer::Copy {
            dest: *dest,
            source: *source,
        }),
        MirInst::CallExtern {
            dest: Some(dest),
            name,
            args,
            ..
        } => Some(ValueProducer::CallExtern {
            dest: *dest,
            name,
            args,
        }),
        MirInst::CallExtern { dest: None, .. } => None,
        MirInst::GetAttr { dest, .. } => Some(ValueProducer::GetAttr { dest: *dest }),
        MirInst::SetAttr { .. } => None,
        MirInst::GetItem { dest, .. } => Some(ValueProducer::GetItem { dest: *dest }),
        MirInst::SetItem { .. } => None,
        MirInst::MakeList { dest, .. } => Some(ValueProducer::MakeList { dest: *dest }),
        MirInst::MakeDict { dest, .. } => Some(ValueProducer::MakeDict { dest: *dest }),
        MirInst::MakeTuple { dest, .. } => Some(ValueProducer::MakeTuple { dest: *dest }),
        MirInst::Raise { .. } => None,
        MirInst::LoadGlobal { dest, .. } => Some(ValueProducer::LoadGlobal { dest: *dest }),
        MirInst::StoreGlobal { .. } => None,
        MirInst::DeleteGlobal { .. } => None,
        MirInst::LoadCell { dest, .. } => Some(ValueProducer::LoadCell { dest: *dest }),
        MirInst::StoreCell { .. } => None,
        MirInst::MakeCell { dest, .. } => Some(ValueProducer::MakeCell { dest: *dest }),
        MirInst::LoadCapture { dest, .. } => Some(ValueProducer::LoadCapture { dest: *dest }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::SymbolId;
    use crate::types::TypeContext;

    #[test]
    fn mixed_int_extern_contracts_preserve_exact_owner_source() {
        let mixed = Some(ReturnAbi::new(
            PhysicalReturn::RawOrBoxedInt,
            ReturnOwnership::ProvenanceTransfer,
        ));

        for name in [
            "mb_pow_int",
            "mb_bigint_add",
            "mb_bigint_sub",
            "mb_bigint_mul",
        ] {
            assert_eq!(
                ExternCompanionContract::for_call(name, &[VReg(10), VReg(11)], MirType::I64, mixed),
                ExternCompanionContract::FreshResultOrNone,
                "{name}",
            );
        }
        for name in ["mb_unbox_int_if_boxed", "mb_unbox_inline_int_if_boxed"] {
            assert_eq!(
                ExternCompanionContract::for_call(name, &[VReg(27)], MirType::I64, mixed),
                ExternCompanionContract::ArgumentPassThroughOrNone { source: VReg(27) },
                "{name}",
            );
        }
        assert_eq!(
            ExternCompanionContract::for_call("explicit_owner_out", &[], MirType::I64, mixed),
            ExternCompanionContract::ExplicitOwnerOut,
        );
        assert_eq!(
            ExternCompanionContract::for_call(
                "dynamic",
                &[],
                MirType::I64,
                Some(ReturnAbi::new(
                    PhysicalReturn::Unknown,
                    ReturnOwnership::ProvenanceTransfer,
                )),
            ),
            ExternCompanionContract::DeferredRuntimeReturn,
        );
    }

    #[test]
    fn declared_argument_contract_fails_closed_when_argument_is_missing() {
        assert_eq!(
            ExternCompanionContract::for_call(
                "mb_unbox_int_if_boxed",
                &[],
                MirType::I64,
                Some(ReturnAbi::new(
                    PhysicalReturn::RawOrBoxedInt,
                    ReturnOwnership::ProvenanceTransfer,
                )),
            ),
            ExternCompanionContract::MissingArgument { index: 0 },
        );
    }

    #[test]
    fn every_value_producing_mir_inst_has_an_owner_action() {
        let tcx = TypeContext::new();
        let ty = tcx.any();
        let d = |n| VReg(n);
        let instructions = vec![
            MirInst::BinOp {
                dest: d(0),
                op: MirBinOp::Add,
                lhs: d(90),
                rhs: d(91),
                ty,
            },
            MirInst::CheckedAdd {
                dest: d(1),
                lhs: d(90),
                rhs: d(91),
                ty,
            },
            MirInst::CheckedSub {
                dest: d(2),
                lhs: d(90),
                rhs: d(91),
                ty,
            },
            MirInst::CheckedMul {
                dest: d(3),
                lhs: d(90),
                rhs: d(91),
                ty,
            },
            MirInst::UnaryOp {
                dest: d(4),
                op: MirUnaryOp::Pos,
                operand: d(90),
                ty,
            },
            MirInst::LoadConst {
                dest: d(5),
                value: super::super::MirConst::Int(1),
                ty,
            },
            MirInst::Call {
                dest: Some(d(6)),
                func: SymbolId(1),
                args: vec![],
                ty,
            },
            MirInst::Copy {
                dest: d(7),
                source: d(90),
            },
            MirInst::CallExtern {
                dest: Some(d(8)),
                name: "unknown".into(),
                args: vec![],
                ty,
            },
            MirInst::GetAttr {
                dest: d(9),
                object: d(90),
                attr: "x".into(),
                ty,
            },
            MirInst::GetItem {
                dest: d(10),
                object: d(90),
                index: d(91),
                ty,
            },
            MirInst::MakeList {
                dest: d(11),
                elements: vec![],
                ty,
            },
            MirInst::MakeDict {
                dest: d(12),
                keys: vec![],
                values: vec![],
                ty,
            },
            MirInst::MakeTuple {
                dest: d(13),
                elements: vec![],
                ty,
            },
            MirInst::LoadGlobal {
                dest: d(14),
                name: SymbolId(2),
                ty,
            },
            MirInst::LoadCell {
                dest: d(15),
                cell_idx: 0,
                ty,
            },
            MirInst::MakeCell {
                dest: d(16),
                value: d(90),
                ty,
            },
            MirInst::LoadCapture {
                dest: d(17),
                capture_idx: 0,
                ty,
            },
        ];
        let expected = HashSet::from([
            ValueProducerKind::BinOp,
            ValueProducerKind::CheckedAdd,
            ValueProducerKind::CheckedSub,
            ValueProducerKind::CheckedMul,
            ValueProducerKind::UnaryOp,
            ValueProducerKind::LoadConst,
            ValueProducerKind::Call,
            ValueProducerKind::Copy,
            ValueProducerKind::CallExtern,
            ValueProducerKind::GetAttr,
            ValueProducerKind::GetItem,
            ValueProducerKind::MakeList,
            ValueProducerKind::MakeDict,
            ValueProducerKind::MakeTuple,
            ValueProducerKind::LoadGlobal,
            ValueProducerKind::LoadCell,
            ValueProducerKind::MakeCell,
            ValueProducerKind::LoadCapture,
        ]);
        let actual = instructions
            .iter()
            .map(|inst| {
                let producer = value_producer(inst).expect("fixture must produce a value");
                let metadata = instruction_owner(inst, &HashMap::new())
                    .expect("every value producer must have an owner action");
                assert_eq!(metadata.dest, producer.dest());
                producer.kind()
            })
            .collect::<HashSet<_>>();

        assert_eq!(actual, expected);
        assert_eq!(actual.len(), instructions.len());
    }
}
