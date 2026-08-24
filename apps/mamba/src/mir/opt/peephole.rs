use crate::mir::*;
use crate::types::TypeContext;
use std::collections::{HashMap, HashSet};

pub fn run_peephole(body: &mut MirBody, _tcx: &TypeContext) -> bool {
    let mut changed = false;

    // 1. Fold jump-to-jump basic blocks (Goto(A) -> Goto(B))
    let mut redirect: HashMap<BlockId, BlockId> = HashMap::new();
    for block in &body.blocks {
        if block.stmts.is_empty() {
            if let Terminator::Goto(target) = block.terminator {
                if target != block.id {
                    redirect.insert(block.id, target);
                }
            }
        }
    }

    // Resolve chained redirects (A -> B -> C)
    let keys: Vec<BlockId> = redirect.keys().copied().collect();
    for k in keys {
        let mut target = redirect[&k];
        let mut visited = HashSet::new();
        visited.insert(k);
        while let Some(&next) = redirect.get(&target) {
            if visited.contains(&next) {
                break;
            }
            visited.insert(next);
            target = next;
        }
        redirect.insert(k, target);
    }

    if !redirect.is_empty() {
        for block in &mut body.blocks {
            match block.terminator {
                Terminator::Goto(target) => {
                    if let Some(&new_target) = redirect.get(&target) {
                        if new_target != target {
                            block.terminator = Terminator::Goto(new_target);
                            changed = true;
                        }
                    }
                }
                Terminator::Branch {
                    cond,
                    then_block,
                    else_block,
                } => {
                    let new_then = redirect.get(&then_block).copied().unwrap_or(then_block);
                    let new_else = redirect.get(&else_block).copied().unwrap_or(else_block);
                    if new_then != then_block || new_else != else_block {
                        block.terminator = Terminator::Branch {
                            cond,
                            then_block: new_then,
                            else_block: new_else,
                        };
                        changed = true;
                    }
                }
                _ => {}
            }
        }
    }

    // 2. Eliminate unreachable basic blocks
    if !body.blocks.is_empty() {
        let entry_id = body.blocks[0].id;
        let mut reachable = HashSet::new();
        let mut stack = vec![entry_id];
        let block_map: HashMap<BlockId, &BasicBlock> = body.blocks.iter().map(|b| (b.id, b)).collect();

        while let Some(bid) = stack.pop() {
            if reachable.insert(bid) {
                if let Some(b) = block_map.get(&bid) {
                    match b.terminator {
                        Terminator::Goto(t) => {
                            if !reachable.contains(&t) {
                                stack.push(t);
                            }
                        }
                        Terminator::Branch {
                            then_block,
                            else_block,
                            ..
                        } => {
                            if !reachable.contains(&then_block) {
                                stack.push(then_block);
                            }
                            if !reachable.contains(&else_block) {
                                stack.push(else_block);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if reachable.len() < body.blocks.len() {
            body.blocks.retain(|b| reachable.contains(&b.id));
            changed = true;
        }
    }

    // 3. Dead Code Elimination for LoadConst instructions without uses
    let mut use_counts: HashMap<VReg, usize> = HashMap::new();
    for block in &body.blocks {
        for inst in &block.stmts {
            collect_inst_uses(inst, &mut use_counts);
        }
        collect_terminator_uses(&block.terminator, &mut use_counts);
    }

    for block in &mut body.blocks {
        let mut new_stmts = Vec::with_capacity(block.stmts.len());
        for inst in block.stmts.drain(..) {
            if let MirInst::LoadConst { dest, .. } = inst {
                if use_counts.get(&dest).copied().unwrap_or(0) == 0 {
                    changed = true;
                    continue;
                }
            }
            new_stmts.push(inst);
        }
        block.stmts = new_stmts;
    }

    changed
}

fn collect_inst_uses(inst: &MirInst, counts: &mut HashMap<VReg, usize>) {
    let mut add_use = |v: VReg| {
        *counts.entry(v).or_insert(0) += 1;
    };

    match inst {
        MirInst::BinOp { lhs, rhs, .. }
        | MirInst::CheckedAdd { lhs, rhs, .. }
        | MirInst::CheckedSub { lhs, rhs, .. }
        | MirInst::CheckedMul { lhs, rhs, .. } => {
            add_use(*lhs);
            add_use(*rhs);
        }
        MirInst::UnaryOp { operand, .. } => {
            add_use(*operand);
        }
        MirInst::Copy { source, .. } => {
            add_use(*source);
        }
        MirInst::Call { args, .. } => {
            for arg in args {
                add_use(*arg);
            }
        }
        MirInst::CallExtern { args, .. } => {
            for arg in args {
                add_use(*arg);
            }
        }
        MirInst::GetAttr { object, .. } => {
            add_use(*object);
        }
        MirInst::SetAttr { object, value, .. } => {
            add_use(*object);
            add_use(*value);
        }
        MirInst::GetItem { object, index, .. } => {
            add_use(*object);
            add_use(*index);
        }
        MirInst::SetItem { object, index, value } => {
            add_use(*object);
            add_use(*index);
            add_use(*value);
        }
        MirInst::MakeList { elements, .. } | MirInst::MakeTuple { elements, .. } => {
            for elem in elements {
                add_use(*elem);
            }
        }
        MirInst::MakeDict { keys, values, .. } => {
            for k in keys {
                add_use(*k);
            }
            for v in values {
                add_use(*v);
            }
        }
        MirInst::Raise { value } => {
            if let Some(v) = value {
                add_use(*v);
            }
        }
        MirInst::StoreGlobal { value, .. } => {
            add_use(*value);
        }
        MirInst::StoreCell { value, .. } => {
            add_use(*value);
        }
        MirInst::MakeCell { value, .. } => {
            add_use(*value);
        }
        MirInst::LoadConst { .. }
        | MirInst::LoadGlobal { .. }
        | MirInst::DeleteGlobal { .. }
        | MirInst::LoadCell { .. }
        | MirInst::LoadCapture { .. } => {}
    }
}

fn collect_terminator_uses(term: &Terminator, counts: &mut HashMap<VReg, usize>) {
    match term {
        Terminator::Return(Some(v)) => {
            *counts.entry(*v).or_insert(0) += 1;
        }
        Terminator::Branch { cond, .. } => {
            *counts.entry(*cond).or_insert(0) += 1;
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::SymbolId;
    use crate::types::TypeContext;

    #[test]
    fn test_peephole_goto_redirect() {
        let tcx = TypeContext::new();
        let b0 = BlockId(0);
        let b1 = BlockId(1);
        let b2 = BlockId(2);

        let mut body = MirBody {
            name: SymbolId(1),
            params: vec![],
            return_ty: tcx.int(),
            blocks: vec![
                BasicBlock {
                    id: b0,
                    stmts: vec![],
                    terminator: Terminator::Goto(b1),
                },
                BasicBlock {
                    id: b1,
                    stmts: vec![],
                    terminator: Terminator::Goto(b2),
                },
                BasicBlock {
                    id: b2,
                    stmts: vec![],
                    terminator: Terminator::Return(None),
                },
            ],
        };

        let changed = run_peephole(&mut body, &tcx);
        assert!(changed);

        // b0 should now jump directly to b2
        assert_eq!(body.blocks[0].terminator, Terminator::Goto(b2));
    }

    #[test]
    fn test_peephole_unreachable_block_elimination() {
        let tcx = TypeContext::new();
        let b0 = BlockId(0);
        let b_dead = BlockId(1);

        let mut body = MirBody {
            name: SymbolId(2),
            params: vec![],
            return_ty: tcx.int(),
            blocks: vec![
                BasicBlock {
                    id: b0,
                    stmts: vec![],
                    terminator: Terminator::Return(None),
                },
                BasicBlock {
                    id: b_dead,
                    stmts: vec![],
                    terminator: Terminator::Return(None),
                },
            ],
        };

        assert_eq!(body.blocks.len(), 2);
        let changed = run_peephole(&mut body, &tcx);
        assert!(changed);
        // Only b0 remains
        assert_eq!(body.blocks.len(), 1);
        assert_eq!(body.blocks[0].id, b0);
    }
}


