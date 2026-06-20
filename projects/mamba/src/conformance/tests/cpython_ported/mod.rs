//! CPython 3.12 ported conformance tests for Mamba (#759).
//!
//! Each submodule corresponds to a CPython test file under `Lib/test/`.
//! Tests are ported from the CPython 3.12.0 tag (commit a6cb7e5d45):
//!   https://github.com/python/cpython/tree/v3.12.0
//!
//! The helper `jit_capture` and `assert_output` mirror the pattern from
//! `tests/cpython/core/generators/mod.rs` — kept self-contained to avoid
//! cross-binary dependencies.
//!
//! Run with:
//!   cargo test -p mamba --test conformance_set cpython_ported::
//!
//! @issue #759

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

const TEST_TIMEOUT_SECS: u64 = 10;

/// Run Python source through the full JIT pipeline, capturing stdout.
/// Acquires JIT_LOCK to serialize across concurrent test threads.
pub fn jit_capture(src: &str) -> String {
    // Use unwrap_or_else to recover from a poisoned lock (caused by a previous
    // test thread panic). The JIT pipeline is stateless across calls so recovery
    // is safe here — the only shared state is JitModule finalization ordering.
    let _jit_guard = JIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    if !errors.is_empty() {
        panic!(
            "type errors: {:?}",
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        );
    }

    // Lowering, codegen, and execution must all run on the SAME thread.
    //
    // The HIR→MIR lowerer and the Cranelift backend register user-defined
    // variadic (`*args`) / kwargs (`**kwargs`) functions into the *thread-local*
    // registries `VARIADIC_SYMBOL_IDS` / `VARIADIC_FUNC_ADDRS` / `KWARGS_*`
    // (see runtime::module). At call time `mb_call_spread` consults those
    // registries via `is_variadic_func` to decide whether to pack arguments
    // into the single args-list parameter the JIT compiled for such functions.
    //
    // The previous harness lowered + codegen'd on the test's main thread but
    // executed the entry on a freshly spawned thread, whose thread-local
    // registries were empty. `is_variadic_func` then returned false on the
    // executor thread, so `mb_call_spread` invoked a `*args` wrapper with the
    // wrong (unpacked) calling convention — an ABI mismatch that silently
    // derailed execution and produced empty stdout. `mamba run` never hit this
    // because it lowers, codegens, and executes all on one thread.
    //
    // Doing the whole pipeline inside the spawned thread keeps execution under
    // the timeout guard while ensuring the registries the executor reads are
    // the same ones lowering + codegen populated.
    let (tx, rx) = mpsc::sync_channel(1);

    let handle = thread::spawn(move || {
        let hir = match lower_module(&module, &checker) {
            Ok(hir) => hir,
            Err(_) => {
                let _ = tx.send(Err("HIR lowering failed".to_string()));
                return;
            }
        };
        let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

        let mut backend = match CraneliftJitBackend::new() {
            Ok(b) => b,
            Err(e) => {
                let _ = tx.send(Err(format!("JIT init failed: {e}")));
                return;
            }
        };
        let output = match backend.codegen(&mir, &checker.tcx) {
            Ok(o) => o,
            Err(e) => {
                let _ = tx.send(Err(format!("JIT codegen failed: {e}")));
                return;
            }
        };

        let CodegenOutput::Jit { entry } = output else {
            let _ = tx.send(Err("expected JIT output".to_string()));
            return;
        };

        let prev = begin_capture();
        let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry) };
        let _result = main_fn();
        cleanup_all_runtime_state();
        let captured = end_capture(prev);
        let _ = tx.send(Ok(captured));
    });

    let result = match rx.recv_timeout(Duration::from_secs(TEST_TIMEOUT_SECS)) {
        Ok(Ok(captured)) => captured,
        Ok(Err(msg)) => panic!("{msg}"),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("JIT execution timed out after {TEST_TIMEOUT_SECS}s");
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("JIT execution thread panicked");
        }
    };

    let _ = handle.join();
    result
}

/// Assert captured stdout equals `expected` (trailing newlines ignored).
pub fn assert_output(actual: &str, expected: &str) {
    let actual_trimmed = actual.trim_end();
    let expected_trimmed = expected.trim_end();
    if actual_trimmed != expected_trimmed {
        let a_lines: Vec<&str> = actual_trimmed.lines().collect();
        let e_lines: Vec<&str> = expected_trimmed.lines().collect();
        let max = a_lines.len().max(e_lines.len());
        let mut diff = String::new();
        for i in 0..max {
            let a = a_lines.get(i).copied().unwrap_or("<missing>");
            let e = e_lines.get(i).copied().unwrap_or("<missing>");
            if a != e {
                diff.push_str(&format!(
                    "  line {}: expected {:?}, got {:?}\n",
                    i + 1,
                    e,
                    a
                ));
            }
        }
        panic!(
            "output mismatch:\n--- expected ---\n{expected_trimmed}\n--- actual ---\n{actual_trimmed}\n--- diff ---\n{diff}"
        );
    }
}

/// Assert that captured stdout contains a substring.
pub fn assert_contains(actual: &str, needle: &str) {
    if !actual.contains(needle) {
        panic!("output missing substring {needle:?}:\n--- actual ---\n{actual}");
    }
}

pub mod test_abc;
pub mod test_any_all;
pub mod test_arith_helpers;
pub mod test_arithmetic_builtins;
pub mod test_arithmetic_mixed;
pub mod test_assert;
pub mod test_assign_unpack;
pub mod test_attribute_access;
pub mod test_aug_assign;
pub mod test_base64;
pub mod test_binascii;
pub mod test_bisect;
pub mod test_bitwise_ops;
pub mod test_bool;
pub mod test_bool_logic_ops;
pub mod test_builtins;
pub mod test_bytearray;
pub mod test_bytes;
pub mod test_bytes_methods;
pub mod test_calendar;
pub mod test_call_signatures;
pub mod test_chained_comparison;
pub mod test_chr_ord;
pub mod test_class;
pub mod test_class_basic;
pub mod test_class_inheritance;
pub mod test_classmethod;
pub mod test_closures_basic;
pub mod test_collections;
pub mod test_compare_membership;
pub mod test_complex;
pub mod test_comprehensions;
pub mod test_container_copy;
pub mod test_copy;
pub mod test_custom_iter;
pub mod test_decorators;
pub mod test_default_kw_args;
pub mod test_dict;
pub mod test_dict_comprehension;
pub mod test_dict_counter;
pub mod test_dict_grouping;
pub mod test_dict_iteration;
pub mod test_dict_keyerror;
pub mod test_dict_methods;
pub mod test_dict_pop_clear;
pub mod test_dict_update_merge;
pub mod test_dict_views;
pub mod test_difflib;
pub mod test_dunder_ops;
pub mod test_elif_chains;
pub mod test_else_clauses;
pub mod test_enumerate;
pub mod test_exception_variants;
pub mod test_exceptions;
pub mod test_filter_map;
pub mod test_float;
pub mod test_float_extended;
pub mod test_float_ops;
pub mod test_fnmatch;
pub mod test_format;
pub mod test_format_braces;
pub mod test_format_method;
pub mod test_fstring;
pub mod test_fstring_expressions;
pub mod test_fstring_format_specs;
pub mod test_func_return_types;
pub mod test_function_elif_return;
pub mod test_functions_as_values;
pub mod test_functools;
pub mod test_gc;
pub mod test_generator_basic;
pub mod test_global_nonlocal;
pub mod test_hashlib;
pub mod test_heapq;
pub mod test_if_elif_else;
pub mod test_int;
pub mod test_int_arithmetic;
pub mod test_int_repr_radix;
pub mod test_isinstance_issubclass;
pub mod test_isinstance_variants;
pub mod test_iter;
pub mod test_iter_protocol;
pub mod test_itertools;
pub mod test_join_over_iterables;
pub mod test_join_variants;
pub mod test_json;
pub mod test_lambda;
pub mod test_list;
pub mod test_list_index_count;
pub mod test_list_methods;
pub mod test_list_mutation_ops;
pub mod test_list_slice_variants;
pub mod test_listcomp_variants;
pub mod test_loop_control;
pub mod test_loop_else_clauses;
pub mod test_match_statement;
pub mod test_math;
pub mod test_max_min_variants;
pub mod test_nested_functions;
pub mod test_nested_listcomp;
pub mod test_nested_loops;
pub mod test_none_identity;
pub mod test_numeric_reductions;
pub mod test_operator;
pub mod test_ord_chr_math;
pub mod test_pct_format;
pub mod test_pickle;
pub mod test_print;
pub mod test_print_kwargs;
pub mod test_print_specials;
pub mod test_property;
pub mod test_queue;
pub mod test_random;
pub mod test_range;
pub mod test_range_enumerate;
pub mod test_range_steps;
pub mod test_range_variants;
pub mod test_re;
pub mod test_recursion_patterns;
pub mod test_repr_str;
pub mod test_reversed_sorted;
pub mod test_round_sum;
pub mod test_scopes;
pub mod test_secrets;
pub mod test_set;
pub mod test_set_binary_ops;
pub mod test_set_methods;
pub mod test_set_ops;
pub mod test_short_circuit_not;
pub mod test_slice;
pub mod test_slice_full_forms;
pub mod test_slicing;
pub mod test_sort_reverse_variants;
pub mod test_starred_unpack;
pub mod test_statistics;
pub mod test_str;
pub mod test_str_capitalize_etc;
pub mod test_str_case_strip;
pub mod test_str_compose;
pub mod test_str_concat_repeat;
pub mod test_str_extended;
pub mod test_str_index_rfind;
pub mod test_str_iteration;
pub mod test_str_methods_basic;
pub mod test_str_padding;
pub mod test_str_partition;
pub mod test_str_predicates;
pub mod test_str_split_variants;
pub mod test_str_unicode_basic;
pub mod test_string_module;
pub mod test_string_slicing_edges;
pub mod test_struct_variadic;
pub mod test_textwrap;
pub mod test_time_module;
pub mod test_truthiness_falsiness;
pub mod test_try_except_variants;
pub mod test_tuple;
pub mod test_tuple_basic;
pub mod test_tuple_ops;
pub mod test_type_conversion;
pub mod test_typing;
pub mod test_unicodedata;
pub mod test_unpack;
pub mod test_uuid;
pub mod test_varargs_kwargs;
pub mod test_walrus;
pub mod test_while_basic;
pub mod test_with;
pub mod test_zip;
pub mod test_zip_dict;
pub mod test_zlib;
