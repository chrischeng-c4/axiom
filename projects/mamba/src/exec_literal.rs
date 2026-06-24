use crate::parser::ast::{CallArg, Expr, Param, Stmt};

#[derive(Clone)]
pub(crate) struct ExecLiteralFnDef {
    pub name: String,
    pub params: Vec<Param>,
}

/// Extract the signature from a narrow module-level form:
///
///     exec("def name(...): ...", globals())
///
/// This is intentionally not a general exec evaluator. It only exposes enough
/// static signature shape for CPython-style argument binding errors that occur
/// before the dynamic function body would execute.
pub(crate) fn global_literal_exec_fn_def(stmt: &Stmt) -> Option<ExecLiteralFnDef> {
    let Stmt::ExprStmt(expr) = stmt else {
        return None;
    };
    let Expr::Call { func, args } = &expr.node else {
        return None;
    };
    if !matches!(&func.node, Expr::Ident(name) if name == "exec") {
        return None;
    }
    if args.len() != 2 {
        return None;
    }
    let CallArg::Positional(code_expr) = &args[0] else {
        return None;
    };
    let Expr::StrLit(source) = &code_expr.node else {
        return None;
    };
    let CallArg::Positional(scope_expr) = &args[1] else {
        return None;
    };
    if !is_globals_call(&scope_expr.node) {
        return None;
    }

    let module = parse_exec_source(source)?;
    if module.stmts.len() != 1 {
        return None;
    }
    let Stmt::FnDef {
        name,
        params,
        decorators,
        ..
    } = &module.stmts[0].node
    else {
        return None;
    };
    if !decorators.is_empty() {
        return None;
    }
    Some(ExecLiteralFnDef {
        name: name.clone(),
        params: params.clone(),
    })
}

fn is_globals_call(expr: &Expr) -> bool {
    let Expr::Call { func, args } = expr else {
        return false;
    };
    args.is_empty() && matches!(&func.node, Expr::Ident(name) if name == "globals")
}

fn parse_exec_source(source: &str) -> Option<crate::parser::ast::Module> {
    use crate::lexer;
    use crate::parser::Parser;
    use crate::source::SourceMap;

    let mut source_map = SourceMap::new();
    let file_id = source_map.add_file("<exec-literal>".to_string(), source.to_string());
    let tokens = lexer::lex(source, file_id);
    let mut parser = Parser::new(tokens, source, file_id);
    parser.skip_newlines();
    parser.parse_module().ok()
}
