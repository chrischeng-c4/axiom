use super::{MirBody, MirInst, Terminator, VReg};
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
    let literal_aliases = propagate_copy_aliases(body, &literal_kinds);
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
            classify_inst_uses(inst, &literal_aliases, &mut analysis);
        }
        classify_terminator_uses(&block.terminator, &literal_aliases, &mut analysis);
    }

    analysis
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

fn propagate_copy_aliases(
    body: &MirBody,
    literal_kinds: &HashMap<VReg, LiteralEscapeKind>,
) -> HashMap<VReg, VReg> {
    let mut literal_aliases: HashMap<VReg, VReg> = literal_kinds
        .keys()
        .copied()
        .map(|vreg| (vreg, vreg))
        .collect();

    loop {
        let mut changed = false;
        for block in &body.blocks {
            for inst in &block.stmts {
                if let MirInst::Copy { dest, source } = inst {
                    if literal_kinds.contains_key(dest) {
                        continue;
                    }
                    if let Some(&root) = literal_aliases.get(source) {
                        if literal_aliases.insert(*dest, root) != Some(root) {
                            changed = true;
                        }
                    }
                }
            }
        }
        if !changed {
            return literal_aliases;
        }
    }
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
