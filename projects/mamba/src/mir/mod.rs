use crate::resolve::SymbolId;
use crate::types::TypeId;

/// A virtual register in SSA form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VReg(pub u32);

/// A basic block identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

/// MIR body — CFG-based representation for a single function.
#[derive(Debug, Clone)]
pub struct MirBody {
    pub name: SymbolId,
    pub params: Vec<(VReg, TypeId)>,
    pub return_ty: TypeId,
    pub blocks: Vec<BasicBlock>,
}

/// A basic block containing a sequence of instructions and a terminator.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub stmts: Vec<MirInst>,
    pub terminator: Terminator,
}

/// MIR instruction.
#[derive(Debug, Clone)]
pub enum MirInst {
    /// dest = lhs op rhs
    BinOp {
        dest: VReg,
        op: MirBinOp,
        lhs: VReg,
        rhs: VReg,
        ty: TypeId,
    },
    /// dest = overflow-checked integer add; promotes to BigInt heap object on 48-bit overflow.
    CheckedAdd {
        dest: VReg,
        lhs: VReg,
        rhs: VReg,
        ty: TypeId,
    },
    /// dest = overflow-checked integer subtract; promotes to BigInt heap object on 48-bit overflow.
    CheckedSub {
        dest: VReg,
        lhs: VReg,
        rhs: VReg,
        ty: TypeId,
    },
    /// dest = overflow-checked integer multiply; promotes to BigInt heap object on 48-bit overflow.
    CheckedMul {
        dest: VReg,
        lhs: VReg,
        rhs: VReg,
        ty: TypeId,
    },
    /// dest = op operand
    UnaryOp {
        dest: VReg,
        op: MirUnaryOp,
        operand: VReg,
        ty: TypeId,
    },
    /// dest = constant
    LoadConst {
        dest: VReg,
        value: MirConst,
        ty: TypeId,
    },
    /// dest = call func(args)
    Call {
        dest: Option<VReg>,
        func: SymbolId,
        args: Vec<VReg>,
        ty: TypeId,
    },
    /// dest = source (copy/move)
    Copy { dest: VReg, source: VReg },
    /// dest = extern_call name(args) — FFI call (#262)
    CallExtern {
        dest: Option<VReg>,
        name: String,
        args: Vec<VReg>,
        ty: TypeId,
    },
    /// dest = object.attr
    GetAttr {
        dest: VReg,
        object: VReg,
        attr: String,
        ty: TypeId,
    },
    /// object.attr = value
    SetAttr {
        object: VReg,
        attr: String,
        value: VReg,
    },
    /// dest = object[index]
    GetItem {
        dest: VReg,
        object: VReg,
        index: VReg,
        ty: TypeId,
    },
    /// object[index] = value
    SetItem {
        object: VReg,
        index: VReg,
        value: VReg,
    },
    /// dest = [elements...]
    MakeList {
        dest: VReg,
        elements: Vec<VReg>,
        ty: TypeId,
    },
    /// dest = {keys: values}
    MakeDict {
        dest: VReg,
        keys: Vec<VReg>,
        values: Vec<VReg>,
        ty: TypeId,
    },
    /// dest = (elements...)
    MakeTuple {
        dest: VReg,
        elements: Vec<VReg>,
        ty: TypeId,
    },
    /// raise value
    Raise { value: Option<VReg> },
    /// dest = GLOBAL_NAMESPACE[name] — load from module-level namespace
    LoadGlobal {
        dest: VReg,
        name: SymbolId,
        ty: TypeId,
    },
    /// GLOBAL_NAMESPACE[name] = value — store to module-level namespace
    StoreGlobal { name: SymbolId, value: VReg },
    /// del GLOBAL_NAMESPACE[name] — remove from module-level namespace
    DeleteGlobal { name: SymbolId },
    /// dest = cells[cell_idx].get() — load through cell indirection (nonlocal)
    LoadCell {
        dest: VReg,
        cell_idx: u32,
        ty: TypeId,
    },
    /// cells[cell_idx].set(value) — store through cell indirection (nonlocal)
    StoreCell { cell_idx: u32, value: VReg },
    /// dest = new_cell(value) — allocate a heap cell initialized with value
    MakeCell { dest: VReg, value: VReg, ty: TypeId },
    /// dest = closure.captures[capture_idx] — load captured cell from closure env
    LoadCapture {
        dest: VReg,
        capture_idx: u32,
        ty: TypeId,
    },
}

/// Block terminator.
#[derive(Debug, Clone)]
pub enum Terminator {
    Return(Option<VReg>),
    Goto(BlockId),
    Branch {
        cond: VReg,
        then_block: BlockId,
        else_block: BlockId,
    },
    Unreachable,
}

#[derive(Debug, Clone, Copy)]
pub enum MirBinOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
    Is,
    IsNot,
    In,
    NotIn,
}

impl MirBinOp {
    /// Convert to opcode integer for runtime dunder dispatch (matches BINOP_DUNDERS).
    pub fn to_opcode(self) -> i64 {
        self as i64
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MirUnaryOp {
    Pos,
    Neg,
    Not,
    BitNot,
}

impl MirUnaryOp {
    /// Convert to opcode integer for runtime dunder dispatch (matches UNARYOP_DUNDERS).
    pub fn to_opcode(self) -> i64 {
        self as i64
    }
}

#[derive(Debug, Clone)]
pub enum MirConst {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Bytes(Vec<u8>),
    None,
    /// Python `NotImplemented` singleton — returned from rich comparison dunders.
    NotImplemented,
    /// Python `Ellipsis` singleton (`...`).
    Ellipsis,
    /// Address of a compiled function (#313 R1 — async body fn pointer).
    FuncRef(crate::resolve::SymbolId),
    /// Address of a runtime extern function (e.g. "mb_abs", "mb_str").
    /// Used when builtin functions are passed as values to higher-order fns.
    ExternFuncRef(String),
}

/// MIR type mapping for codegen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirType {
    I8,
    I32,
    I64,
    F32,
    F64,
    Ptr,
    Void,
}

/// Extern function declaration for FFI imports (#261).
#[derive(Debug, Clone)]
pub struct MirExtern {
    /// Mangled/C name of the function
    pub name: String,
    /// Parameter types in C ABI
    pub params: Vec<MirType>,
    /// Return type in C ABI
    pub return_type: MirType,
    /// Which dynamic library contains this function
    pub lib_name: String,
}

/// A complete MIR module with functions and extern declarations.
#[derive(Debug, Clone, Default)]
pub struct MirModule {
    pub bodies: Vec<MirBody>,
    pub externs: Vec<MirExtern>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::SymbolId;
    use crate::types::TypeContext;

    mod escape_analysis_gate;

    #[test]
    fn test_vreg_equality_and_hash() {
        use std::collections::HashSet;
        let v0 = VReg(0);
        let v0_dup = VReg(0);
        let v1 = VReg(1);
        assert_eq!(v0, v0_dup);
        assert_ne!(v0, v1);
        let mut set = HashSet::new();
        set.insert(v0);
        assert!(set.contains(&v0_dup));
        assert!(!set.contains(&v1));
    }

    #[test]
    fn test_block_id_equality_and_hash() {
        use std::collections::HashSet;
        let b0 = BlockId(0);
        let b1 = BlockId(1);
        assert_eq!(b0, BlockId(0));
        assert_ne!(b0, b1);
        let mut set = HashSet::new();
        set.insert(b0);
        assert!(set.contains(&BlockId(0)));
    }

    #[test]
    fn test_mir_binop_to_opcode_unique() {
        // Every MirBinOp variant should map to a distinct opcode
        let ops = [
            MirBinOp::Add,
            MirBinOp::Sub,
            MirBinOp::Mul,
            MirBinOp::Div,
            MirBinOp::FloorDiv,
            MirBinOp::Mod,
            MirBinOp::Pow,
            MirBinOp::Eq,
            MirBinOp::NotEq,
            MirBinOp::Lt,
            MirBinOp::Gt,
            MirBinOp::LtEq,
            MirBinOp::GtEq,
            MirBinOp::And,
            MirBinOp::Or,
            MirBinOp::BitAnd,
            MirBinOp::BitOr,
            MirBinOp::BitXor,
            MirBinOp::LShift,
            MirBinOp::RShift,
            MirBinOp::Is,
            MirBinOp::IsNot,
            MirBinOp::In,
            MirBinOp::NotIn,
        ];
        let opcodes: Vec<i64> = ops.iter().map(|op| op.to_opcode()).collect();
        let unique: std::collections::HashSet<i64> = opcodes.iter().copied().collect();
        assert_eq!(opcodes.len(), unique.len(), "opcodes must be unique");
    }

    #[test]
    fn test_mir_unaryop_to_opcode_unique() {
        let ops = [
            MirUnaryOp::Pos,
            MirUnaryOp::Neg,
            MirUnaryOp::Not,
            MirUnaryOp::BitNot,
        ];
        let opcodes: Vec<i64> = ops.iter().map(|op| op.to_opcode()).collect();
        let unique: std::collections::HashSet<i64> = opcodes.iter().copied().collect();
        assert_eq!(opcodes.len(), unique.len(), "opcodes must be unique");
    }

    #[test]
    fn test_mir_const_variants() {
        let int_c = MirConst::Int(42);
        assert!(matches!(int_c, MirConst::Int(42)));
        let float_c = MirConst::Float(3.14);
        assert!(matches!(float_c, MirConst::Float(f) if (f - 3.14).abs() < 1e-10));
        let bool_c = MirConst::Bool(true);
        assert!(matches!(bool_c, MirConst::Bool(true)));
        let str_c = MirConst::Str("hello".to_string());
        assert!(matches!(str_c, MirConst::Str(ref s) if s == "hello"));
        let none_c = MirConst::None;
        assert!(matches!(none_c, MirConst::None));
        let func_c = MirConst::FuncRef(SymbolId(7));
        assert!(matches!(func_c, MirConst::FuncRef(SymbolId(7))));
    }

    #[test]
    fn test_mir_type_equality() {
        assert_eq!(MirType::I64, MirType::I64);
        assert_ne!(MirType::I32, MirType::I64);
        assert_ne!(MirType::Void, MirType::Ptr);
        assert_eq!(MirType::F64, MirType::F64);
    }

    #[test]
    fn test_mir_module_default() {
        let module = MirModule::default();
        assert!(module.bodies.is_empty());
        assert!(module.externs.is_empty());
    }

    #[test]
    fn test_mir_body_construction() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let body = MirBody {
            name: SymbolId(0),
            params: vec![(VReg(0), int_ty), (VReg(1), int_ty)],
            return_ty: int_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![MirInst::BinOp {
                    dest: VReg(2),
                    op: MirBinOp::Add,
                    lhs: VReg(0),
                    rhs: VReg(1),
                    ty: int_ty,
                }],
                terminator: Terminator::Return(Some(VReg(2))),
            }],
        };
        assert_eq!(body.params.len(), 2);
        assert_eq!(body.blocks.len(), 1);
        assert_eq!(body.blocks[0].stmts.len(), 1);
    }

    #[test]
    fn test_terminator_variants() {
        let ret = Terminator::Return(Some(VReg(0)));
        assert!(matches!(ret, Terminator::Return(Some(VReg(0)))));
        let ret_none = Terminator::Return(None);
        assert!(matches!(ret_none, Terminator::Return(None)));
        let goto = Terminator::Goto(BlockId(5));
        assert!(matches!(goto, Terminator::Goto(BlockId(5))));
        let branch = Terminator::Branch {
            cond: VReg(0),
            then_block: BlockId(1),
            else_block: BlockId(2),
        };
        assert!(matches!(branch, Terminator::Branch { .. }));
        let unreach = Terminator::Unreachable;
        assert!(matches!(unreach, Terminator::Unreachable));
    }

    #[test]
    fn test_mir_extern_construction() {
        let ext = MirExtern {
            name: "puts".to_string(),
            params: vec![MirType::Ptr],
            return_type: MirType::I32,
            lib_name: "libc".to_string(),
        };
        assert_eq!(ext.name, "puts");
        assert_eq!(ext.params.len(), 1);
        assert_eq!(ext.return_type, MirType::I32);
    }
}
