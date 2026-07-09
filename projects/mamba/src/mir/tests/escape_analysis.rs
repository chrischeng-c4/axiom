#![cfg(test)]

use super::super::*;
use crate::resolve::SymbolId;
use crate::types::TypeContext;

fn body(stmts: Vec<MirInst>, terminator: Terminator) -> MirBody {
    let tcx = TypeContext::new();
    MirBody {
        name: SymbolId(0),
        params: vec![],
        return_ty: tcx.none(),
        blocks: vec![BasicBlock {
            id: BlockId(0),
            stmts,
            terminator,
        }],
    }
}

#[test]
fn local_list_and_dict_literals_stay_non_escaping() {
    let tcx = TypeContext::new();
    let body = body(
        vec![
            MirInst::MakeList {
                dest: VReg(0),
                elements: vec![],
                ty: tcx.any(),
            },
            MirInst::MakeDict {
                dest: VReg(1),
                keys: vec![],
                values: vec![],
                ty: tcx.any(),
            },
        ],
        Terminator::Return(None),
    );

    let analysis = analyze_literal_escapes(&body);

    assert_eq!(
        analysis.classification(VReg(0)),
        Some(LiteralEscapeClassification::NonEscaping)
    );
    assert_eq!(
        analysis.classification(VReg(1)),
        Some(LiteralEscapeClassification::NonEscaping)
    );
}

#[test]
fn returned_literal_is_escaping() {
    let tcx = TypeContext::new();
    let body = body(
        vec![MirInst::MakeList {
            dest: VReg(0),
            elements: vec![],
            ty: tcx.any(),
        }],
        Terminator::Return(Some(VReg(0))),
    );

    let analysis = analyze_literal_escapes(&body);

    assert_eq!(
        analysis.classification(VReg(0)),
        Some(LiteralEscapeClassification::Escaping)
    );
}

#[test]
fn global_cell_and_store_sites_are_escaping() {
    let tcx = TypeContext::new();
    let body = body(
        vec![
            MirInst::MakeList {
                dest: VReg(0),
                elements: vec![],
                ty: tcx.any(),
            },
            MirInst::StoreGlobal {
                name: SymbolId(7),
                value: VReg(0),
            },
            MirInst::MakeDict {
                dest: VReg(1),
                keys: vec![],
                values: vec![],
                ty: tcx.any(),
            },
            MirInst::StoreCell {
                cell_idx: 0,
                value: VReg(1),
            },
            MirInst::MakeList {
                dest: VReg(2),
                elements: vec![],
                ty: tcx.any(),
            },
            MirInst::SetAttr {
                object: VReg(9),
                attr: "payload".to_string(),
                value: VReg(2),
            },
            MirInst::MakeDict {
                dest: VReg(3),
                keys: vec![],
                values: vec![],
                ty: tcx.any(),
            },
            MirInst::SetItem {
                object: VReg(8),
                index: VReg(7),
                value: VReg(3),
            },
        ],
        Terminator::Return(None),
    );

    let analysis = analyze_literal_escapes(&body);

    for reg in [VReg(0), VReg(1), VReg(2), VReg(3)] {
        assert_eq!(
            analysis.classification(reg),
            Some(LiteralEscapeClassification::Escaping)
        );
    }
}

#[test]
fn call_arguments_are_escaping() {
    let tcx = TypeContext::new();
    let body = body(
        vec![
            MirInst::MakeList {
                dest: VReg(0),
                elements: vec![],
                ty: tcx.any(),
            },
            MirInst::Call {
                dest: None,
                func: SymbolId(99),
                args: vec![VReg(0)],
                ty: tcx.any(),
            },
            MirInst::MakeDict {
                dest: VReg(1),
                keys: vec![],
                values: vec![],
                ty: tcx.any(),
            },
            MirInst::CallExtern {
                dest: None,
                name: "ffi_consume".to_string(),
                args: vec![VReg(1)],
                ty: tcx.any(),
            },
        ],
        Terminator::Return(None),
    );

    let analysis = analyze_literal_escapes(&body);

    assert_eq!(
        analysis.classification(VReg(0)),
        Some(LiteralEscapeClassification::Escaping)
    );
    assert_eq!(
        analysis.classification(VReg(1)),
        Some(LiteralEscapeClassification::Escaping)
    );
}

#[test]
fn unsupported_uses_default_to_escaping() {
    let tcx = TypeContext::new();
    let body = body(
        vec![MirInst::MakeList {
            dest: VReg(0),
            elements: vec![],
            ty: tcx.any(),
        }],
        Terminator::Branch {
            cond: VReg(0),
            then_block: BlockId(1),
            else_block: BlockId(2),
        },
    );

    let analysis = analyze_literal_escapes(&body);

    assert_eq!(
        analysis.classification(VReg(0)),
        Some(LiteralEscapeClassification::Escaping)
    );
}

#[test]
fn copy_aliases_follow_root_literal_classification() {
    let tcx = TypeContext::new();
    let body = body(
        vec![
            MirInst::MakeDict {
                dest: VReg(0),
                keys: vec![],
                values: vec![],
                ty: tcx.any(),
            },
            MirInst::Copy {
                dest: VReg(1),
                source: VReg(0),
            },
            MirInst::Copy {
                dest: VReg(2),
                source: VReg(1),
            },
        ],
        Terminator::Return(Some(VReg(2))),
    );

    let analysis = analyze_literal_escapes(&body);

    assert_eq!(
        analysis.classification(VReg(0)),
        Some(LiteralEscapeClassification::Escaping)
    );
}

#[test]
fn copy_alias_propagation_does_not_rewrite_literal_roots() {
    let tcx = TypeContext::new();
    let body = body(
        vec![
            MirInst::MakeList {
                dest: VReg(0),
                elements: vec![],
                ty: tcx.any(),
            },
            MirInst::MakeDict {
                dest: VReg(1),
                keys: vec![],
                values: vec![],
                ty: tcx.any(),
            },
            MirInst::Copy {
                dest: VReg(1),
                source: VReg(0),
            },
        ],
        Terminator::Return(None),
    );

    let analysis = analyze_literal_escapes(&body);

    assert_eq!(
        analysis.classification(VReg(0)),
        Some(LiteralEscapeClassification::NonEscaping)
    );
    assert_eq!(
        analysis.classification(VReg(1)),
        Some(LiteralEscapeClassification::NonEscaping)
    );
}
