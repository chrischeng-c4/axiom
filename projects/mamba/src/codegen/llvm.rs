/// LLVM backend for AOT compilation (#305).
///
/// Generates LLVM IR from MIR, then compiles to native object files
/// using the LLVM C API. This backend enables full AOT compilation
/// with LLVM's optimization passes.
///
/// Current status: structural implementation with textual LLVM IR
/// generation. Direct LLVM-C FFI can be added when llvm-sys dependency
/// is enabled.
use crate::codegen::{CodegenBackend, CodegenOutput};
use crate::mir::*;
use crate::types::{Ty, TypeContext};

/// LLVM backend that generates LLVM IR textually, then invokes
/// the system LLVM tools (llc, clang) for compilation.
pub struct LlvmBackend {
    opt_level: OptLevel,
    target_triple: String,
}

/// Optimization level for LLVM.
#[derive(Debug, Clone, Copy)]
pub enum OptLevel {
    O0,
    O1,
    O2,
    O3,
}

impl LlvmBackend {
    pub fn new() -> Self {
        Self {
            opt_level: OptLevel::O2,
            target_triple: default_target_triple(),
        }
    }

    pub fn with_opt(mut self, level: OptLevel) -> Self {
        self.opt_level = level;
        self
    }

    pub fn with_target(mut self, triple: String) -> Self {
        self.target_triple = triple;
        self
    }
}

impl CodegenBackend for LlvmBackend {
    fn codegen(
        &mut self,
        module: &MirModule,
        tcx: &TypeContext,
    ) -> crate::error::Result<CodegenOutput> {
        let ir = generate_llvm_ir(module, tcx, &self.target_triple);

        // Write IR to temp file and invoke LLC
        let tmp = std::env::temp_dir().join("mamba_output.ll");
        std::fs::write(&tmp, &ir).map_err(|e| crate::error::MambaError::codegen(e.to_string()))?;

        let obj_path = std::env::temp_dir().join("mamba_output.o");
        let opt_flag = match self.opt_level {
            OptLevel::O0 => "-O0",
            OptLevel::O1 => "-O1",
            OptLevel::O2 => "-O2",
            OptLevel::O3 => "-O3",
        };

        // Try to invoke llc
        let status = std::process::Command::new("llc")
            .args([
                opt_flag,
                "--filetype=obj",
                "-o",
                obj_path.to_str().unwrap(),
                tmp.to_str().unwrap(),
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                let bytes = std::fs::read(&obj_path)
                    .map_err(|e| crate::error::MambaError::codegen(e.to_string()))?;
                Ok(CodegenOutput::ObjectFile(bytes))
            }
            Ok(s) => Err(crate::error::MambaError::codegen(format!(
                "llc exited with status {s}"
            ))),
            Err(_) => {
                // LLC not available — return IR text for downstream use
                Ok(CodegenOutput::LlvmIr(ir))
            }
        }
    }

    fn name(&self) -> &str {
        "llvm"
    }
}

// ── LLVM IR Generation ──

/// Generate LLVM IR text from a MIR module.
fn generate_llvm_ir(module: &MirModule, tcx: &TypeContext, target: &str) -> String {
    let mut ir = String::new();
    ir.push_str(&format!("; ModuleID = 'mamba'\n"));
    ir.push_str(&format!("target triple = \"{target}\"\n\n"));

    // Declare external runtime functions
    for ext in &module.externs {
        let ret_ty = mir_type_to_llvm(&ext.return_type);
        let params: Vec<String> = ext
            .params
            .iter()
            .map(|p| mir_type_to_llvm(p).to_string())
            .collect();
        ir.push_str(&format!(
            "declare {ret_ty} @{}({})\n",
            ext.name,
            params.join(", ")
        ));
    }
    ir.push('\n');

    // Generate function bodies
    for body in &module.bodies {
        generate_function(&mut ir, body, tcx);
    }

    ir
}

/// Generate LLVM IR for a single MIR function.
fn generate_function(ir: &mut String, body: &MirBody, tcx: &TypeContext) {
    let name = if body.name.0 == u32::MAX {
        "__main__".to_string()
    } else {
        format!("fn_{}", body.name.0)
    };

    let ret_ty = mir_type_to_llvm_from_type_id(body.return_ty, tcx);
    let params: Vec<String> = body
        .params
        .iter()
        .map(|(vreg, ty)| format!("{} %v{}", mir_type_to_llvm_from_type_id(*ty, tcx), vreg.0))
        .collect();

    ir.push_str(&format!(
        "define {ret_ty} @{name}({}) {{\n",
        params.join(", ")
    ));

    for block in &body.blocks {
        ir.push_str(&format!("bb{}:\n", block.id.0));

        for inst in &block.stmts {
            generate_inst(ir, inst, tcx);
        }

        generate_terminator(ir, &block.terminator);
    }

    ir.push_str("}\n\n");
}

/// Generate LLVM IR for a single MIR instruction.
fn generate_inst(ir: &mut String, inst: &MirInst, tcx: &TypeContext) {
    match inst {
        MirInst::LoadConst { dest, value, .. } => {
            match value {
                MirConst::Int(i) => {
                    ir.push_str(&format!("  %v{} = add i64 0, {i}\n", dest.0));
                }
                MirConst::BigInt(s) => {
                    ir.push_str(&format!(
                        "  ; bigint const: {}\n  %v{} = add i64 0, {}\n",
                        s.escape_debug(),
                        dest.0,
                        crate::runtime::bigint_ops::bigint_immortal_from_literal(s).to_bits()
                    ));
                }
                MirConst::Float(f) => {
                    let bits = f.to_bits();
                    ir.push_str(&format!(
                        "  %v{} = bitcast i64 {} to double\n",
                        dest.0, bits
                    ));
                }
                MirConst::Bool(b) => {
                    ir.push_str(&format!(
                        "  %v{} = add i64 0, {}\n",
                        dest.0,
                        if *b { 1 } else { 0 }
                    ));
                }
                MirConst::Str(s) => {
                    // String constants require global data — simplified to i64 placeholder
                    ir.push_str(&format!(
                        "  ; string const: \"{}\"\n  %v{} = add i64 0, 0\n",
                        s.escape_debug(),
                        dest.0
                    ));
                }
                MirConst::Bytes(data) => {
                    ir.push_str(&format!(
                        "  ; bytes const ({} bytes)\n  %v{} = add i64 0, 0\n",
                        data.len(),
                        dest.0
                    ));
                }
                MirConst::None => {
                    ir.push_str(&format!("  %v{} = add i64 0, 0\n", dest.0));
                }
                MirConst::NotImplemented => {
                    ir.push_str(&format!(
                        "  %v{} = add i64 0, {}\n",
                        dest.0,
                        crate::runtime::MbValue::not_implemented().to_bits()
                    ));
                }
                MirConst::Ellipsis => {
                    ir.push_str(&format!(
                        "  %v{} = add i64 0, {}\n",
                        dest.0,
                        crate::runtime::MbValue::ellipsis().to_bits()
                    ));
                }
                MirConst::FuncRef(sym) => {
                    // Load function pointer as i64 for async body (#313 R1)
                    let fname = if sym.0 == u32::MAX {
                        "__main__".to_string()
                    } else {
                        format!("fn_{}", sym.0)
                    };
                    ir.push_str(&format!("  %v{} = ptrtoint ptr @{fname} to i64\n", dest.0));
                }
                MirConst::ExternFuncRef(name) => {
                    // Load address of a runtime extern function.
                    ir.push_str(&format!("  %v{} = ptrtoint ptr @{name} to i64\n", dest.0));
                }
            }
        }
        MirInst::Copy { dest, source } => {
            ir.push_str(&format!("  %v{} = add i64 0, %v{}\n", dest.0, source.0));
        }
        MirInst::BinOp {
            dest,
            op,
            lhs,
            rhs,
            ty,
        } => {
            let resolved_ty = tcx.get(*ty);
            let use_primitive = match op {
                MirBinOp::Is | MirBinOp::IsNot => true,
                MirBinOp::In | MirBinOp::NotIn | MirBinOp::Pow => false,
                _ => matches!(resolved_ty, Ty::Int | Ty::Float | Ty::Bool),
            };
            if matches!(op, MirBinOp::In | MirBinOp::NotIn) {
                // `in`/`not in`: call mb_obj_contains(rhs, lhs) — RHS is the container
                ir.push_str(&format!(
                    "  %contains_{d} = call i64 @mb_obj_contains(i64 %v{r}, i64 %v{l})\n",
                    d = dest.0,
                    r = rhs.0,
                    l = lhs.0
                ));
                if matches!(op, MirBinOp::NotIn) {
                    ir.push_str(&format!("  %v{d} = xor i64 %contains_{d}, 1\n", d = dest.0));
                } else {
                    ir.push_str(&format!("  %v{d} = add i64 0, %contains_{d}\n", d = dest.0));
                }
            } else if !use_primitive {
                let opcode = op.to_opcode();
                ir.push_str(&format!(
                    "  %v{} = call i64 @mb_dispatch_binop(i64 {}, i64 %v{}, i64 %v{})\n",
                    dest.0, opcode, lhs.0, rhs.0
                ));
            } else {
                let op_str = match op {
                    MirBinOp::Add => "add",
                    MirBinOp::Sub => "sub",
                    MirBinOp::Mul => "mul",
                    MirBinOp::Div | MirBinOp::FloorDiv => "sdiv",
                    MirBinOp::Mod => "srem",
                    MirBinOp::And | MirBinOp::BitAnd => "and",
                    MirBinOp::Or | MirBinOp::BitOr => "or",
                    MirBinOp::BitXor => "xor",
                    MirBinOp::LShift => "shl",
                    MirBinOp::RShift => "ashr",
                    // Comparison ops handled in the match below; others routed to runtime above
                    MirBinOp::Eq
                    | MirBinOp::NotEq
                    | MirBinOp::Lt
                    | MirBinOp::Gt
                    | MirBinOp::LtEq
                    | MirBinOp::GtEq
                    | MirBinOp::Is
                    | MirBinOp::IsNot => "add", // placeholder; handled below
                    // Pow, In, NotIn are already routed to runtime dispatch above
                    MirBinOp::Pow | MirBinOp::In | MirBinOp::NotIn => {
                        unreachable!("should be handled by runtime dispatch above")
                    }
                };
                match op {
                    MirBinOp::Eq
                    | MirBinOp::NotEq
                    | MirBinOp::Lt
                    | MirBinOp::Gt
                    | MirBinOp::LtEq
                    | MirBinOp::GtEq
                    | MirBinOp::Is
                    | MirBinOp::IsNot => {
                        let cmp = match op {
                            MirBinOp::Eq | MirBinOp::Is => "eq",
                            MirBinOp::NotEq | MirBinOp::IsNot => "ne",
                            MirBinOp::Lt => "slt",
                            MirBinOp::Gt => "sgt",
                            MirBinOp::LtEq => "sle",
                            MirBinOp::GtEq => "sge",
                            _ => "eq",
                        };
                        ir.push_str(&format!(
                            "  %cmp_{d} = icmp {cmp} i64 %v{l}, %v{r}\n  %v{d} = zext i1 %cmp_{d} to i64\n",
                            d = dest.0, l = lhs.0, r = rhs.0
                        ));
                    }
                    _ => {
                        ir.push_str(&format!(
                            "  %v{} = {op_str} i64 %v{}, %v{}\n",
                            dest.0, lhs.0, rhs.0
                        ));
                    }
                }
            }
        }
        MirInst::UnaryOp {
            dest,
            op,
            operand,
            ty,
        } => {
            let resolved_ty = tcx.get(*ty);
            let is_primitive = matches!(resolved_ty, Ty::Int | Ty::Float | Ty::Bool);
            if !is_primitive {
                let opcode = op.to_opcode();
                ir.push_str(&format!(
                    "  %v{} = call i64 @mb_dispatch_unaryop(i64 {}, i64 %v{})\n",
                    dest.0, opcode, operand.0
                ));
            } else {
                match op {
                    MirUnaryOp::Neg => {
                        ir.push_str(&format!("  %v{} = sub i64 0, %v{}\n", dest.0, operand.0));
                    }
                    MirUnaryOp::Not => {
                        ir.push_str(&format!("  %v{} = xor i64 %v{}, 1\n", dest.0, operand.0));
                    }
                    MirUnaryOp::BitNot => {
                        ir.push_str(&format!("  %v{} = xor i64 %v{}, -1\n", dest.0, operand.0));
                    }
                    MirUnaryOp::Pos => {
                        ir.push_str(&format!("  %v{} = add i64 0, %v{}\n", dest.0, operand.0));
                    }
                }
            }
        }
        MirInst::Call {
            dest, func, args, ..
        } => {
            let arg_list: Vec<String> = args.iter().map(|a| format!("i64 %v{}", a.0)).collect();
            if let Some(d) = dest {
                ir.push_str(&format!(
                    "  %v{} = call i64 @fn_{}({})\n",
                    d.0,
                    func.0,
                    arg_list.join(", ")
                ));
            } else {
                ir.push_str(&format!(
                    "  call void @fn_{}({})\n",
                    func.0,
                    arg_list.join(", ")
                ));
            }
        }
        MirInst::CallExtern {
            dest, name, args, ..
        } => {
            let arg_list: Vec<String> = args.iter().map(|a| format!("i64 %v{}", a.0)).collect();
            if let Some(d) = dest {
                ir.push_str(&format!(
                    "  %v{} = call i64 @{name}({})\n",
                    d.0,
                    arg_list.join(", ")
                ));
            } else {
                ir.push_str(&format!("  call void @{name}({})\n", arg_list.join(", ")));
            }
        }
        MirInst::MakeList { dest, elements, .. } => {
            ir.push_str(&format!("  ; make list ({} elems)\n", elements.len()));
            ir.push_str(&format!("  %v{} = call i64 @mb_list_new()\n", dest.0));
            for e in elements {
                ir.push_str(&format!(
                    "  call void @mb_list_append(i64 %v{}, i64 %v{})\n",
                    dest.0, e.0
                ));
            }
        }
        MirInst::MakeTuple { dest, elements, .. } => {
            ir.push_str(&format!(
                "  ; make tuple ({} elems)\n  %v{} = add i64 0, 0\n",
                elements.len(),
                dest.0
            ));
        }
        MirInst::MakeDict { dest, .. } => {
            ir.push_str(&format!("  %v{} = call i64 @mb_dict_new()\n", dest.0));
        }
        MirInst::GetAttr {
            dest, object, attr, ..
        } => {
            ir.push_str(&format!(
                "  ; getattr(.{attr})\n  %v{} = call i64 @mb_getattr(i64 %v{}, i64 0)\n",
                dest.0, object.0
            ));
        }
        MirInst::GetItem {
            dest,
            object,
            index,
            ty,
        } => {
            let resolved_ty = tcx.get(*ty);
            let is_list = matches!(resolved_ty, Ty::List(_));
            let func_name = if is_list {
                "mb_list_getitem"
            } else {
                "mb_obj_getitem"
            };
            ir.push_str(&format!(
                "  %v{} = call i64 @{func_name}(i64 %v{}, i64 %v{})\n",
                dest.0, object.0, index.0
            ));
        }
        MirInst::Raise { value } => {
            if let Some(v) = value {
                ir.push_str(&format!("  call void @mb_raise(i64 %v{}, i64 0)\n", v.0));
            }
        }
        MirInst::SetAttr {
            object,
            attr,
            value,
        } => {
            ir.push_str(&format!(
                "  ; setattr(.{attr})\n  call void @mb_setattr(i64 %v{}, i64 0, i64 %v{})\n",
                object.0, value.0
            ));
        }
        MirInst::SetItem {
            object,
            index,
            value,
        } => {
            ir.push_str(&format!(
                "  call void @mb_list_setitem(i64 %v{}, i64 %v{}, i64 %v{})\n",
                object.0, index.0, value.0
            ));
        }
        MirInst::LoadGlobal { dest, name, .. } => {
            ir.push_str(&format!(
                "  %v{} = call i64 @mb_global_get_id(i64 {})\n",
                dest.0, name.0
            ));
        }
        MirInst::StoreGlobal { name, value } => {
            ir.push_str(&format!(
                "  call void @mb_global_set_id(i64 {}, i64 %v{})\n",
                name.0, value.0
            ));
        }
        MirInst::DeleteGlobal { name } => {
            ir.push_str(&format!("  call void @mb_global_del_id(i64 {})\n", name.0));
        }
        MirInst::LoadCell { dest, cell_idx, .. } => {
            ir.push_str(&format!(
                "  %v{} = call i64 @mb_cell_get(i64 {})\n",
                dest.0, cell_idx
            ));
        }
        MirInst::StoreCell { cell_idx, value } => {
            ir.push_str(&format!(
                "  call void @mb_cell_set(i64 {}, i64 %v{})\n",
                cell_idx, value.0
            ));
        }
        MirInst::MakeCell { dest, value, .. } => {
            ir.push_str(&format!(
                "  %v{} = call i64 @mb_cell_new(i64 %v{})\n",
                dest.0, value.0
            ));
        }
        MirInst::LoadCapture {
            dest, capture_idx, ..
        } => {
            ir.push_str(&format!(
                "  %v{} = call i64 @mb_closure_get_capture(i64 %v0, i64 {})\n",
                dest.0, capture_idx
            ));
        }
        // LLVM backend: fall back to wrapping arithmetic (no BigInt promotion).
        MirInst::CheckedAdd { dest, lhs, rhs, .. } => {
            ir.push_str(&format!(
                "  %v{} = add i64 %v{}, %v{}\n",
                dest.0, lhs.0, rhs.0
            ));
        }
        MirInst::CheckedSub { dest, lhs, rhs, .. } => {
            ir.push_str(&format!(
                "  %v{} = sub i64 %v{}, %v{}\n",
                dest.0, lhs.0, rhs.0
            ));
        }
        MirInst::CheckedMul { dest, lhs, rhs, .. } => {
            ir.push_str(&format!(
                "  %v{} = mul i64 %v{}, %v{}\n",
                dest.0, lhs.0, rhs.0
            ));
        }
    }
}

/// Generate LLVM IR for a block terminator.
fn generate_terminator(ir: &mut String, term: &Terminator) {
    match term {
        Terminator::Return(Some(vreg)) => {
            ir.push_str(&format!("  ret i64 %v{}\n", vreg.0));
        }
        Terminator::Return(None) => {
            ir.push_str("  ret void\n");
        }
        Terminator::Goto(block) => {
            ir.push_str(&format!("  br label %bb{}\n", block.0));
        }
        Terminator::Branch {
            cond,
            then_block,
            else_block,
        } => {
            ir.push_str(&format!(
                "  %br_{c} = icmp ne i64 %v{c}, 0\n  br i1 %br_{c}, label %bb{t}, label %bb{e}\n",
                c = cond.0,
                t = then_block.0,
                e = else_block.0
            ));
        }
        Terminator::Unreachable => {
            ir.push_str("  unreachable\n");
        }
    }
}

fn mir_type_to_llvm(ty: &MirType) -> &'static str {
    match ty {
        MirType::I8 => "i8",
        MirType::I32 => "i32",
        MirType::I64 => "i64",
        MirType::F32 => "float",
        MirType::F64 => "double",
        MirType::Ptr => "ptr",
        MirType::Void => "void",
    }
}

fn mir_type_to_llvm_from_type_id(ty: crate::types::TypeId, tcx: &TypeContext) -> &'static str {
    use crate::types::Ty;
    match tcx.get(ty) {
        Ty::None | Ty::Never => "void",
        Ty::Float => "double",
        _ => "i64",
    }
}

fn default_target_triple() -> String {
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else {
        "x86_64" // fallback
    };

    if cfg!(target_os = "macos") {
        format!("{arch}-apple-darwin")
    } else if cfg!(target_os = "linux") {
        format!("{arch}-unknown-linux-gnu")
    } else {
        format!("{arch}-unknown-unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypeContext;

    #[test]
    fn test_generate_simple_ir() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(u32::MAX),
                params: Vec::new(),
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(0),
                        value: MirConst::Int(42),
                        ty: int_ty,
                    }],
                    terminator: Terminator::Return(Some(VReg(0))),
                }],
            }],
            externs: Vec::new(),
        };

        let ir = generate_llvm_ir(&module, &tcx, "arm64-apple-darwin");
        assert!(ir.contains("define i64 @__main__"));
        assert!(ir.contains("add i64 0, 42"));
        assert!(ir.contains("ret i64 %v0"));
    }

    #[test]
    fn test_generate_binop_ir() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(0),
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
            }],
            externs: Vec::new(),
        };

        let ir = generate_llvm_ir(&module, &tcx, "arm64-apple-darwin");
        assert!(ir.contains("add i64 %v0, %v1"));
    }

    #[test]
    fn test_mir_type_to_llvm_all() {
        assert_eq!(mir_type_to_llvm(&MirType::I8), "i8");
        assert_eq!(mir_type_to_llvm(&MirType::I32), "i32");
        assert_eq!(mir_type_to_llvm(&MirType::I64), "i64");
        assert_eq!(mir_type_to_llvm(&MirType::F32), "float");
        assert_eq!(mir_type_to_llvm(&MirType::F64), "double");
        assert_eq!(mir_type_to_llvm(&MirType::Ptr), "ptr");
        assert_eq!(mir_type_to_llvm(&MirType::Void), "void");
    }

    #[test]
    fn test_mir_type_to_llvm_from_type_id() {
        let tcx = TypeContext::new();
        assert_eq!(mir_type_to_llvm_from_type_id(tcx.none(), &tcx), "void");
        assert_eq!(mir_type_to_llvm_from_type_id(tcx.float(), &tcx), "double");
        assert_eq!(mir_type_to_llvm_from_type_id(tcx.int(), &tcx), "i64");
        assert_eq!(mir_type_to_llvm_from_type_id(tcx.bool(), &tcx), "i64");
        assert_eq!(mir_type_to_llvm_from_type_id(tcx.str(), &tcx), "i64");
    }

    #[test]
    fn test_default_target_triple_not_empty() {
        let triple = default_target_triple();
        assert!(!triple.is_empty());
        // Should contain a known arch
        assert!(
            triple.contains("arm64") || triple.contains("x86_64"),
            "unexpected triple: {triple}"
        );
    }

    #[test]
    fn test_generate_float_const() {
        let tcx = TypeContext::new();
        let float_ty = tcx.float();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(u32::MAX),
                params: vec![],
                return_ty: float_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(0),
                        value: MirConst::Float(3.14),
                        ty: float_ty,
                    }],
                    terminator: Terminator::Return(Some(VReg(0))),
                }],
            }],
            externs: vec![],
        };
        let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("bitcast i64"));
        assert!(ir.contains("to double"));
    }

    #[test]
    fn test_generate_bool_const() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(u32::MAX),
                params: vec![],
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![
                        MirInst::LoadConst {
                            dest: VReg(0),
                            value: MirConst::Bool(true),
                            ty: int_ty,
                        },
                        MirInst::LoadConst {
                            dest: VReg(1),
                            value: MirConst::Bool(false),
                            ty: int_ty,
                        },
                    ],
                    terminator: Terminator::Return(Some(VReg(0))),
                }],
            }],
            externs: vec![],
        };
        let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("add i64 0, 1"), "true should be 1");
        assert!(ir.contains("add i64 0, 0"), "false should be 0");
    }

    #[test]
    fn test_generate_string_const() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(u32::MAX),
                params: vec![],
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(0),
                        value: MirConst::Str("hello".to_string()),
                        ty: int_ty,
                    }],
                    terminator: Terminator::Return(Some(VReg(0))),
                }],
            }],
            externs: vec![],
        };
        let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("string const: \"hello\""));
    }

    #[test]
    fn test_generate_none_const() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(u32::MAX),
                params: vec![],
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(0),
                        value: MirConst::None,
                        ty: int_ty,
                    }],
                    terminator: Terminator::Return(Some(VReg(0))),
                }],
            }],
            externs: vec![],
        };
        let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("add i64 0, 0"));
    }

    #[test]
    fn test_generate_copy_inst() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(0),
                params: vec![(VReg(0), int_ty)],
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::Copy {
                        dest: VReg(1),
                        source: VReg(0),
                    }],
                    terminator: Terminator::Return(Some(VReg(1))),
                }],
            }],
            externs: vec![],
        };
        let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("%v1 = add i64 0, %v0"));
    }

    #[test]
    fn test_generate_comparison_ops() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let ops_and_cmp = [
            (MirBinOp::Eq, "eq"),
            (MirBinOp::NotEq, "ne"),
            (MirBinOp::Lt, "slt"),
            (MirBinOp::Gt, "sgt"),
            (MirBinOp::LtEq, "sle"),
            (MirBinOp::GtEq, "sge"),
        ];
        for (op, expected_cmp) in ops_and_cmp {
            let module = MirModule {
                bodies: vec![MirBody {
                    name: crate::resolve::SymbolId(0),
                    params: vec![(VReg(0), int_ty), (VReg(1), int_ty)],
                    return_ty: int_ty,
                    blocks: vec![BasicBlock {
                        id: BlockId(0),
                        stmts: vec![MirInst::BinOp {
                            dest: VReg(2),
                            op,
                            lhs: VReg(0),
                            rhs: VReg(1),
                            ty: int_ty,
                        }],
                        terminator: Terminator::Return(Some(VReg(2))),
                    }],
                }],
                externs: vec![],
            };
            let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
            assert!(
                ir.contains(&format!("icmp {expected_cmp}")),
                "expected icmp {expected_cmp} in IR for op {op:?}, got:\n{ir}"
            );
        }
    }

    #[test]
    fn test_generate_terminator_goto() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(u32::MAX),
                params: vec![],
                return_ty: int_ty,
                blocks: vec![
                    BasicBlock {
                        id: BlockId(0),
                        stmts: vec![],
                        terminator: Terminator::Goto(BlockId(1)),
                    },
                    BasicBlock {
                        id: BlockId(1),
                        stmts: vec![MirInst::LoadConst {
                            dest: VReg(0),
                            value: MirConst::Int(0),
                            ty: int_ty,
                        }],
                        terminator: Terminator::Return(Some(VReg(0))),
                    },
                ],
            }],
            externs: vec![],
        };
        let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("br label %bb1"));
    }

    #[test]
    fn test_generate_terminator_branch() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(u32::MAX),
                params: vec![],
                return_ty: int_ty,
                blocks: vec![
                    BasicBlock {
                        id: BlockId(0),
                        stmts: vec![MirInst::LoadConst {
                            dest: VReg(0),
                            value: MirConst::Bool(true),
                            ty: int_ty,
                        }],
                        terminator: Terminator::Branch {
                            cond: VReg(0),
                            then_block: BlockId(1),
                            else_block: BlockId(2),
                        },
                    },
                    BasicBlock {
                        id: BlockId(1),
                        stmts: vec![MirInst::LoadConst {
                            dest: VReg(1),
                            value: MirConst::Int(1),
                            ty: int_ty,
                        }],
                        terminator: Terminator::Return(Some(VReg(1))),
                    },
                    BasicBlock {
                        id: BlockId(2),
                        stmts: vec![MirInst::LoadConst {
                            dest: VReg(2),
                            value: MirConst::Int(0),
                            ty: int_ty,
                        }],
                        terminator: Terminator::Return(Some(VReg(2))),
                    },
                ],
            }],
            externs: vec![],
        };
        let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("icmp ne i64 %v0, 0"));
        assert!(ir.contains("label %bb1"));
        assert!(ir.contains("label %bb2"));
    }

    #[test]
    fn test_generate_terminator_unreachable() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(u32::MAX),
                params: vec![],
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![],
                    terminator: Terminator::Unreachable,
                }],
            }],
            externs: vec![],
        };
        let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("unreachable"));
    }

    #[test]
    fn test_generate_unary_neg() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(u32::MAX),
                params: vec![],
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![
                        MirInst::LoadConst {
                            dest: VReg(0),
                            value: MirConst::Int(5),
                            ty: int_ty,
                        },
                        MirInst::UnaryOp {
                            dest: VReg(1),
                            op: crate::mir::MirUnaryOp::Neg,
                            operand: VReg(0),
                            ty: int_ty,
                        },
                    ],
                    terminator: Terminator::Return(Some(VReg(1))),
                }],
            }],
            externs: vec![],
        };
        let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("sub i64 0, %v0"));
    }

    #[test]
    fn test_generate_unary_not() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(u32::MAX),
                params: vec![],
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![
                        MirInst::LoadConst {
                            dest: VReg(0),
                            value: MirConst::Bool(true),
                            ty: int_ty,
                        },
                        MirInst::UnaryOp {
                            dest: VReg(1),
                            op: crate::mir::MirUnaryOp::Not,
                            operand: VReg(0),
                            ty: int_ty,
                        },
                    ],
                    terminator: Terminator::Return(Some(VReg(1))),
                }],
            }],
            externs: vec![],
        };
        let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("xor i64 %v0, 1"));
    }

    #[test]
    fn test_generate_extern_decl() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(u32::MAX),
                params: vec![],
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(0),
                        value: MirConst::Int(0),
                        ty: int_ty,
                    }],
                    terminator: Terminator::Return(Some(VReg(0))),
                }],
            }],
            externs: vec![MirExtern {
                name: "mb_print".to_string(),
                params: vec![MirType::I64],
                return_type: MirType::Void,
                lib_name: "runtime".to_string(),
            }],
        };
        let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("declare void @mb_print(i64)"));
    }

    #[test]
    fn test_generate_return_void() {
        let tcx = TypeContext::new();
        let none_ty = tcx.none();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(u32::MAX),
                params: vec![],
                return_ty: none_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![],
                    terminator: Terminator::Return(None),
                }],
            }],
            externs: vec![],
        };
        let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("ret void"));
    }

    #[test]
    fn test_llvm_backend_name() {
        let backend = LlvmBackend::new();
        assert_eq!(backend.name(), "llvm");
    }

    #[test]
    fn test_llvm_backend_builder() {
        let backend = LlvmBackend::new()
            .with_opt(OptLevel::O3)
            .with_target("x86_64-unknown-linux-gnu".to_string());
        assert_eq!(backend.name(), "llvm");
    }

    #[test]
    fn test_generate_funcref_const() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let module = MirModule {
            bodies: vec![MirBody {
                name: crate::resolve::SymbolId(u32::MAX),
                params: vec![],
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(0),
                        value: MirConst::FuncRef(crate::resolve::SymbolId(5)),
                        ty: int_ty,
                    }],
                    terminator: Terminator::Return(Some(VReg(0))),
                }],
            }],
            externs: vec![],
        };
        let ir = generate_llvm_ir(&module, &tcx, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("ptrtoint ptr @fn_5 to i64"));
    }
}
