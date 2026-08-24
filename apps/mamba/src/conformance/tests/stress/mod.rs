//! Property-based & Stress test suite for Mamba (#gen12_fuzzing).
//!
//! Probes:
//! - Parser resilience (garbage/malformed inputs, extreme depth, unicode fuzzing)
//! - JIT resilience (stack depth, deep recursion, allocation stress, loop stress)
//! - Type system edge cases (diamond inheritance MRO, __slots__ conflicts, dynamic __bases__ mutation)

#![allow(dead_code)]

use crate::codegen::cranelift::jit::{CraneliftJitBackend, JIT_LOCK};
use crate::codegen::{CodegenBackend, CodegenOutput};
use crate::lower::{lower_hir_to_mir_with_symbols, lower_module};
use crate::parser;
use crate::runtime::cleanup_all_runtime_state;
use crate::runtime::output::{begin_capture, end_capture};
use crate::source::span::FileId;
use crate::types::TypeChecker;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const STRESS_TIMEOUT_SECS: u64 = 10;

/// Execute Python source through Mamba JIT pipeline safely, capturing stdout.
/// Returns `Ok(output)` on success, or `Err(errmsg)` if parse, typecheck, codegen, or execution panics/fails.
pub fn jit_try(src: &str) -> Result<String, String> {
    let _jit_guard = JIT_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let module = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        parser::parse(src, FileId(0))
    })) {
        Ok(Ok(m)) => m,
        Ok(Err(e)) => return Err(format!("parse error: {e}")),
        Err(_) => return Err("parser panicked".to_string()),
    };

    let mut checker = TypeChecker::new();
    let errors = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        checker.check_module(&module)
    })) {
        Ok(errs) => errs,
        Err(_) => return Err("type checker panicked".to_string()),
    };

    if !errors.is_empty() {
        return Err(format!(
            "type errors: {:?}",
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        ));
    }

    let hir = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        lower_module(&module, &checker)
    })) {
        Ok(Ok(h)) => h,
        Ok(Err(e)) => return Err(format!("HIR lowering error: {e:?}")),
        Err(_) => return Err("HIR lowering panicked".to_string()),
    };

    let mir = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols)
    }))
    .map_err(|_| "MIR lowering panicked".to_string())?;

    let mut backend = match CraneliftJitBackend::new() {
        Ok(b) => b,
        Err(e) => return Err(format!("JIT init error: {e}")),
    };

    let output = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        backend.codegen(&mir, &checker.tcx)
    })) {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => return Err(format!("codegen error: {e}")),
        Err(_) => return Err("codegen panicked".to_string()),
    };

    match output {
        CodegenOutput::Jit { entry } => {
            let entry_addr = entry as usize;
            let (tx, rx) = mpsc::sync_channel(1);

            let (sym_info, func_info) = {
                use std::collections::HashMap;
                let mut sym_func_addrs: Vec<(u32, String, *const u8)> = Vec::new();
                let mut sym_names: HashMap<crate::resolve::SymbolId, String> = HashMap::new();
                for s in checker.symbols.all_symbols() {
                    sym_names.insert(s.id, s.name.clone());
                }
                for (id, name) in &hir.sym_names {
                    sym_names.insert(*id, name.clone());
                }
                for f in &hir.functions {
                    if let Some(name) = sym_names.get(&f.name) {
                        if let Some(ptr) = backend.get_func_ptr(f.name.0) {
                            sym_func_addrs.push((f.name.0, name.clone(), ptr));
                        }
                    }
                }
                crate::runtime::module::build_introspection_state(&checker, &hir, &sym_func_addrs)
            };
            let func_info_addrs: Vec<(String, usize)> = func_info
                .into_iter()
                .map(|(name, fv)| (name, fv.to_bits() as usize))
                .collect();

            let handle = thread::spawn(move || {
                crate::runtime::closure::set_module_sym_info(sym_info);
                let func_info_thread: std::collections::HashMap<
                    String,
                    crate::runtime::value::MbValue,
                > = func_info_addrs
                    .into_iter()
                    .map(|(name, bits)| {
                        (name, crate::runtime::value::MbValue::from_bits(bits as u64))
                    })
                    .collect();
                crate::runtime::closure::set_module_func_info(func_info_thread);

                let prev = begin_capture();
                let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry_addr) };
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| main_fn()));
                let pending_exc = if crate::runtime::exception::has_current_exception() {
                    let exc_type =
                        crate::runtime::exception::current_exception_type().unwrap_or_default();
                    let exc_msg =
                        crate::runtime::exception::current_exception_message().unwrap_or_default();
                    Some((exc_type, exc_msg))
                } else {
                    None
                };
                cleanup_all_runtime_state();
                let captured = end_capture(prev);
                let _ = tx.send((res, pending_exc, captured));
            });

            let result = match rx.recv_timeout(Duration::from_secs(STRESS_TIMEOUT_SECS)) {
                Ok((Ok(_), Some((exc_type, exc_msg)), captured)) => {
                    Err(format!("{exc_type}: {exc_msg}; captured output: {captured}"))
                }
                Ok((Ok(_), None, captured)) => Ok(captured),
                Ok((Err(_), _, captured)) => {
                    Err(format!("execution panicked; captured output: {captured}"))
                }
                Err(mpsc::RecvTimeoutError::Timeout) => Err("execution timed out".to_string()),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    Err("execution thread disconnected".to_string())
                }
            };

            let _ = handle.join();
            result
        }
        _ => Err("expected JIT output".to_string()),
    }
}

/// Run Python source through JIT and assert captured stdout matches `expected`.
pub fn jit_assert_output(src: &str, expected: &str) {
    match jit_try(src) {
        Ok(actual) => {
            let actual_trimmed = actual.trim_end();
            let expected_trimmed = expected.trim_end();
            if actual_trimmed != expected_trimmed {
                panic!("output mismatch:\nexpected:\n{expected_trimmed}\nactual:\n{actual_trimmed}");
            }
        }
        Err(e) => panic!("jit_try failed unexpectedly: {e}"),
    }
}

pub mod fail_cases;
pub mod test_jit_resilience;
pub mod test_parser_resilience;
pub mod test_type_system_edge_cases;
