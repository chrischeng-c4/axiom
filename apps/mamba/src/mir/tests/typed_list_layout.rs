#![cfg(test)]

use super::super::*;
use crate::resolve::SymbolId;
use crate::types::{Ty, TypeContext, TypeId};

fn body(tcx: &TypeContext, stmts: Vec<MirInst>, terminator: Terminator) -> MirBody {
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

fn list_ty(tcx: &mut TypeContext, element_ty: TypeId) -> TypeId {
    tcx.intern(Ty::List(element_ty))
}

#[test]
fn scalar_typed_lists_are_reported_in_vreg_order() {
    let mut tcx = TypeContext::new();
    let int_ty = tcx.int();
    let float_ty = tcx.float();
    let list_int = list_ty(&mut tcx, int_ty);
    let list_float = list_ty(&mut tcx, float_ty);
    let mir = body(
        &tcx,
        vec![
            MirInst::MakeList {
                dest: VReg(7),
                elements: vec![],
                ty: list_float,
            },
            MirInst::MakeList {
                dest: VReg(3),
                elements: vec![],
                ty: list_int,
            },
        ],
        Terminator::Return(None),
    );

    let analysis = analyze_typed_list_layouts(&mir, &tcx);

    assert_eq!(analysis.len(), 2);
    assert!(!analysis.is_empty());
    assert_eq!(
        analysis.element_kind(VReg(3)),
        Some(TypedListElementKind::Int)
    );
    assert_eq!(
        analysis.element_kind(VReg(7)),
        Some(TypedListElementKind::Float)
    );
    assert!(analysis.is_eligible(VReg(3)));
    assert!(analysis.is_eligible(VReg(7)));

    let ordered: Vec<_> = analysis
        .iter()
        .map(|(vreg, info)| (vreg, info.element_kind))
        .collect();
    assert_eq!(
        ordered,
        vec![
            (VReg(3), TypedListElementKind::Int),
            (VReg(7), TypedListElementKind::Float),
        ]
    );
}

#[test]
fn non_scalar_lists_and_non_list_make_lists_are_excluded() {
    let mut tcx = TypeContext::new();
    let str_ty = tcx.str();
    let any_ty = tcx.any();
    let list_str = list_ty(&mut tcx, str_ty);
    let list_any = list_ty(&mut tcx, any_ty);
    let mir = body(
        &tcx,
        vec![
            MirInst::MakeList {
                dest: VReg(0),
                elements: vec![],
                ty: list_str,
            },
            MirInst::MakeList {
                dest: VReg(1),
                elements: vec![],
                ty: list_any,
            },
            MirInst::MakeList {
                dest: VReg(2),
                elements: vec![],
                ty: tcx.any(),
            },
            MirInst::MakeTuple {
                dest: VReg(3),
                elements: vec![],
                ty: tcx.any(),
            },
            MirInst::MakeDict {
                dest: VReg(4),
                keys: vec![],
                values: vec![],
                ty: tcx.any(),
            },
        ],
        Terminator::Return(None),
    );

    let analysis = analyze_typed_list_layouts(&mir, &tcx);

    assert!(analysis.is_empty());
    assert_eq!(analysis.get(VReg(0)), None);
    assert_eq!(analysis.get(VReg(1)), None);
    assert_eq!(analysis.get(VReg(2)), None);
    assert_eq!(analysis.get(VReg(3)), None);
    assert_eq!(analysis.get(VReg(4)), None);
}

#[test]
fn escaping_scalar_typed_list_stays_reported_but_not_eligible() {
    let mut tcx = TypeContext::new();
    let int_ty = tcx.int();
    let list_int = list_ty(&mut tcx, int_ty);
    let mir = body(
        &tcx,
        vec![MirInst::MakeList {
            dest: VReg(0),
            elements: vec![],
            ty: list_int,
        }],
        Terminator::Return(Some(VReg(0))),
    );

    let analysis = analyze_typed_list_layouts(&mir, &tcx);

    assert_eq!(analysis.len(), 1);
    assert_eq!(
        analysis.escape_classification(VReg(0)),
        Some(LiteralEscapeClassification::Escaping)
    );
    assert!(!analysis.is_eligible(VReg(0)));
}

#[test]
fn copy_alias_escape_marks_root_candidate_ineligible() {
    let mut tcx = TypeContext::new();
    let float_ty = tcx.float();
    let list_float = list_ty(&mut tcx, float_ty);
    let mir = body(
        &tcx,
        vec![
            MirInst::MakeList {
                dest: VReg(0),
                elements: vec![],
                ty: list_float,
            },
            MirInst::Copy {
                dest: VReg(1),
                source: VReg(0),
            },
        ],
        Terminator::Return(Some(VReg(1))),
    );

    let analysis = analyze_typed_list_layouts(&mir, &tcx);

    assert_eq!(
        analysis.escape_classification(VReg(0)),
        Some(LiteralEscapeClassification::Escaping)
    );
    assert!(!analysis.is_eligible(VReg(0)));
    assert_eq!(analysis.get(VReg(1)), None);
}
