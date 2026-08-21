use super::{MirBody, MirInst, Terminator, VReg};
use crate::types::{Ty, TypeContext, TypeId};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralEscapeKind {
    List,
    Dict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralEscapeClassification {
    Escaping,
    NonEscaping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiteralEscapeInfo {
    pub kind: LiteralEscapeKind,
    pub classification: LiteralEscapeClassification,
}

#[derive(Debug, Clone, Default)]
pub struct LiteralEscapeAnalysis {
    literals: HashMap<VReg, LiteralEscapeInfo>,
}

impl LiteralEscapeAnalysis {
    pub fn get(&self, vreg: VReg) -> Option<&LiteralEscapeInfo> {
        self.literals.get(&vreg)
    }

    pub fn classification(&self, vreg: VReg) -> Option<LiteralEscapeClassification> {
        self.get(vreg).map(|info| info.classification)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&VReg, &LiteralEscapeInfo)> {
        self.literals.iter()
    }

    pub fn len(&self) -> usize {
        self.literals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.literals.is_empty()
    }
}

pub fn analyze_literal_escapes(body: &MirBody) -> LiteralEscapeAnalysis {
    let literal_kinds = collect_literal_kinds(body);
    // Alias map, updated in program order as we walk (see below) rather
    // than precomputed once to a whole-function fixed point. VRegs here
    // are reused as the "current value of this variable" slot across
    // reassignments (not true SSA), so the same Copy-destination VReg can
    // legitimately alias a *different* literal root at different points
    // in the same function (e.g. `x = [1]; x = [2]` both lower through
    // the same scratch Copy destination before each StoreGlobal). A
    // precomputed static map can only remember the last root a
    // destination ever aliased to, which silently steals the escaping
    // mark from an earlier literal that shared the same destination and
    // wrongly leaves it classified NonEscaping — the JIT then elides its
    // GC tracking even though it truly escapes, corrupting runtime state
    // once the object outlives the local scope (#1610). Interleaving the
    // alias update with the escape check for each instruction, in a
    // single forward pass, keys the classification off the alias that
    // was actually live at the point of use.
    let mut current_aliases: HashMap<VReg, VReg> = literal_kinds
        .keys()
        .copied()
        .map(|vreg| (vreg, vreg))
        .collect();
    let mut analysis = LiteralEscapeAnalysis {
        literals: literal_kinds
            .into_iter()
            .map(|(vreg, kind)| {
                (
                    vreg,
                    LiteralEscapeInfo {
                        kind,
                        classification: LiteralEscapeClassification::NonEscaping,
                    },
                )
            })
            .collect(),
    };

    for block in &body.blocks {
        for inst in &block.stmts {
            if let MirInst::Copy { dest, source } = inst {
                if !analysis.literals.contains_key(dest) {
                    match current_aliases.get(source).copied() {
                        Some(root) => {
                            current_aliases.insert(*dest, root);
                        }
                        None => {
                            current_aliases.remove(dest);
                        }
                    }
                }
                continue;
            }
            classify_inst_uses(inst, &current_aliases, &mut analysis);
        }
        classify_terminator_uses(&block.terminator, &current_aliases, &mut analysis);
    }

    analysis
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypedListElementKind {
    Int,
    Float,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypedListLayoutInfo {
    pub element_kind: TypedListElementKind,
    pub escape_classification: LiteralEscapeClassification,
}

impl TypedListLayoutInfo {
    pub fn is_eligible(self) -> bool {
        matches!(
            self.escape_classification,
            LiteralEscapeClassification::NonEscaping
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct TypedListLayoutAnalysis {
    candidates: HashMap<VReg, TypedListLayoutInfo>,
    order: Vec<VReg>,
}

impl TypedListLayoutAnalysis {
    pub fn get(&self, vreg: VReg) -> Option<&TypedListLayoutInfo> {
        self.candidates.get(&vreg)
    }

    pub fn element_kind(&self, vreg: VReg) -> Option<TypedListElementKind> {
        self.get(vreg).map(|info| info.element_kind)
    }

    pub fn escape_classification(&self, vreg: VReg) -> Option<LiteralEscapeClassification> {
        self.get(vreg).map(|info| info.escape_classification)
    }

    pub fn is_eligible(&self, vreg: VReg) -> bool {
        self.get(vreg).is_some_and(|info| info.is_eligible())
    }

    pub fn iter(&self) -> impl Iterator<Item = (VReg, &TypedListLayoutInfo)> {
        let candidates = &self.candidates;
        self.order.iter().copied().map(move |vreg| {
            (
                vreg,
                candidates
                    .get(&vreg)
                    .expect("typed-list candidate order must stay in sync"),
            )
        })
    }

    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }
}

pub fn analyze_typed_list_layouts(body: &MirBody, tcx: &TypeContext) -> TypedListLayoutAnalysis {
    let literal_escapes = analyze_literal_escapes(body);
    let mut candidates = HashMap::new();
    let mut order = Vec::new();

    for block in &body.blocks {
        for inst in &block.stmts {
            let MirInst::MakeList { dest, ty, .. } = inst else {
                continue;
            };
            let Some(element_kind) = scalar_typed_list_element_kind(*ty, tcx) else {
                continue;
            };
            let escape_classification = literal_escapes
                .classification(*dest)
                .unwrap_or(LiteralEscapeClassification::Escaping);
            if candidates
                .insert(
                    *dest,
                    TypedListLayoutInfo {
                        element_kind,
                        escape_classification,
                    },
                )
                .is_none()
            {
                order.push(*dest);
            }
        }
    }

    order.sort_by_key(|vreg| vreg.0);

    TypedListLayoutAnalysis { candidates, order }
}

fn scalar_typed_list_element_kind(ty: TypeId, tcx: &TypeContext) -> Option<TypedListElementKind> {
    match tcx.get(ty) {
        Ty::List(element_ty) if *element_ty == tcx.int() => Some(TypedListElementKind::Int),
        Ty::List(element_ty) if *element_ty == tcx.float() => Some(TypedListElementKind::Float),
        _ => None,
    }
}

fn collect_literal_kinds(body: &MirBody) -> HashMap<VReg, LiteralEscapeKind> {
    let mut literal_kinds = HashMap::new();
    for block in &body.blocks {
        for inst in &block.stmts {
            match inst {
                MirInst::MakeList { dest, .. } => {
                    literal_kinds.insert(*dest, LiteralEscapeKind::List);
                }
                MirInst::MakeDict { dest, .. } => {
                    literal_kinds.insert(*dest, LiteralEscapeKind::Dict);
                }
                _ => {}
            }
        }
    }
    literal_kinds
}

fn classify_inst_uses(
    inst: &MirInst,
    literal_aliases: &HashMap<VReg, VReg>,
    analysis: &mut LiteralEscapeAnalysis,
) {
    match inst {
        MirInst::LoadConst { .. }
        | MirInst::Copy { .. }
        | MirInst::LoadGlobal { .. }
        | MirInst::DeleteGlobal { .. }
        | MirInst::LoadCell { .. }
        | MirInst::LoadCapture { .. } => {}
        MirInst::Call { args, .. } | MirInst::CallExtern { args, .. } => {
            mark_regs_as_escaping(args.iter().copied(), literal_aliases, analysis);
        }
        MirInst::Raise { value } => {
            mark_regs_as_escaping(value.iter().copied(), literal_aliases, analysis);
        }
        MirInst::StoreGlobal { value, .. }
        | MirInst::StoreCell { value, .. }
        | MirInst::MakeCell { value, .. } => {
            mark_reg_as_escaping(*value, literal_aliases, analysis);
        }
        MirInst::SetAttr { object, value, .. } => {
            mark_regs_as_escaping([*object, *value], literal_aliases, analysis);
        }
        MirInst::SetItem {
            object,
            index,
            value,
        } => {
            mark_regs_as_escaping([*object, *index, *value], literal_aliases, analysis);
        }
        MirInst::BinOp { lhs, rhs, .. }
        | MirInst::CheckedAdd { lhs, rhs, .. }
        | MirInst::CheckedSub { lhs, rhs, .. }
        | MirInst::CheckedMul { lhs, rhs, .. } => {
            mark_regs_as_escaping([*lhs, *rhs], literal_aliases, analysis);
        }
        MirInst::UnaryOp { operand, .. } => {
            mark_reg_as_escaping(*operand, literal_aliases, analysis);
        }
        MirInst::GetAttr { object, .. } => {
            mark_reg_as_escaping(*object, literal_aliases, analysis);
        }
        MirInst::GetItem { object, index, .. } => {
            mark_regs_as_escaping([*object, *index], literal_aliases, analysis);
        }
        MirInst::MakeList { elements, .. } | MirInst::MakeTuple { elements, .. } => {
            mark_regs_as_escaping(elements.iter().copied(), literal_aliases, analysis);
        }
        MirInst::MakeDict { keys, values, .. } => {
            mark_regs_as_escaping(keys.iter().copied(), literal_aliases, analysis);
            mark_regs_as_escaping(values.iter().copied(), literal_aliases, analysis);
        }
    }
}

fn classify_terminator_uses(
    terminator: &Terminator,
    literal_aliases: &HashMap<VReg, VReg>,
    analysis: &mut LiteralEscapeAnalysis,
) {
    match terminator {
        Terminator::Return(value) => {
            mark_regs_as_escaping(value.iter().copied(), literal_aliases, analysis);
        }
        Terminator::Branch { cond, .. } => {
            mark_reg_as_escaping(*cond, literal_aliases, analysis);
        }
        Terminator::Goto(_) | Terminator::Unreachable => {}
    }
}

fn mark_regs_as_escaping(
    regs: impl IntoIterator<Item = VReg>,
    literal_aliases: &HashMap<VReg, VReg>,
    analysis: &mut LiteralEscapeAnalysis,
) {
    for reg in regs {
        mark_reg_as_escaping(reg, literal_aliases, analysis);
    }
}

fn mark_reg_as_escaping(
    reg: VReg,
    literal_aliases: &HashMap<VReg, VReg>,
    analysis: &mut LiteralEscapeAnalysis,
) {
    let Some(&root) = literal_aliases.get(&reg) else {
        return;
    };
    if let Some(info) = analysis.literals.get_mut(&root) {
        info.classification = LiteralEscapeClassification::Escaping;
    }
}
