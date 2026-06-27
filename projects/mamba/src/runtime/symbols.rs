/// Runtime symbol registry for JIT execution (#295).
///
/// Maps mb_* function names to their physical addresses and MIR type
/// signatures so the JIT backend can wire them into the executable module.
use crate::mir::{MirExtern, MirType};

/// A runtime symbol: name, function pointer, and ABI signature.
pub struct RuntimeSymbol {
    pub name: &'static str,
    pub addr: *const u8,
    pub params: &'static [MirType],
    pub return_type: MirType,
}

// Safety: function pointers to static functions are Send+Sync.
unsafe impl Send for RuntimeSymbol {}
unsafe impl Sync for RuntimeSymbol {}

/// Helper macro to register a runtime function.
macro_rules! rt_sym {
    ($name:expr, $func:expr, [$($p:expr),*], $ret:expr) => {
        RuntimeSymbol {
            name: $name,
            addr: $func as *const u8,
            params: &[$($p),*],
            return_type: $ret,
        }
    };
}

/// Return all registered runtime symbols.
pub fn runtime_symbols() -> Vec<RuntimeSymbol> {
    use super::async_rt;
    use super::builtins;
    use super::class;
    use super::closure;
    use super::dict_ops;
    use super::exception;
    use super::file_io;
    use super::generator;
    use super::iter;
    use super::list_ops;
    use super::module;
    use super::pep695;
    use super::set_ops;
    use super::stdlib::functools_mod;
    use super::stdlib::traceback_mod;
    use super::string_ops;
    use super::tokio_exec;
    use super::tuple_ops;
    use MirType::*;

    vec![
        // ── Boxing (raw → NaN-boxed MbValue) ──
        rt_sym!(
            "mb_box_int",
            builtins::mb_box_int as fn(i64) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_box_bool",
            builtins::mb_box_bool as fn(i64) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_pow_int",
            builtins::mb_pow_int as fn(i64, i64) -> i64,
            [I64, I64],
            I64
        ),
        RuntimeSymbol {
            name: "mb_pow_float",
            addr: builtins::mb_pow_float as *const u8,
            params: &[MirType::F64, MirType::F64],
            return_type: MirType::F64,
        },
        RuntimeSymbol {
            name: "mb_box_float",
            addr: builtins::mb_box_float as *const u8,
            params: &[MirType::F64],
            return_type: MirType::I64,
        },
        // ── Unboxing (NaN-boxed MbValue → raw primitive) for nested capture bindings (#827) ──
        rt_sym!(
            "mb_unbox_int",
            builtins::mb_unbox_int as fn(super::MbValue) -> i64,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_unbox_bool",
            builtins::mb_unbox_bool as fn(super::MbValue) -> i64,
            [I64],
            I64
        ),
        RuntimeSymbol {
            name: "mb_unbox_float",
            addr: builtins::mb_unbox_float as *const u8,
            params: &[MirType::I64],
            return_type: MirType::F64,
        },
        // ── Smart unbox: passes through if already raw, unboxes if NaN-tagged ──
        // Used in entry-body lowering's typed-return path for top-level call
        // results that may be either raw (literal arms) or boxed (IfExpr / getattr).
        rt_sym!(
            "mb_unbox_int_if_boxed",
            builtins::mb_unbox_int_if_boxed as fn(super::MbValue) -> i64,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_unbox_bool_if_boxed",
            builtins::mb_unbox_bool_if_boxed as fn(super::MbValue) -> i64,
            [I64],
            I64
        ),
        RuntimeSymbol {
            name: "mb_unbox_float_if_boxed",
            addr: builtins::mb_unbox_float_if_boxed as *const u8,
            params: &[MirType::I64],
            return_type: MirType::F64,
        },
        // ── Builtins ──
        rt_sym!(
            "mb_print",
            builtins::mb_print as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_print_args",
            builtins::mb_print_args as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_is_none",
            builtins::mb_is_none as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_is_not_none",
            builtins::mb_is_not_none as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_is_identity",
            builtins::mb_is_identity as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_is_not_identity",
            builtins::mb_is_not_identity as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_len",
            builtins::mb_len as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_int",
            builtins::mb_int as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_float",
            builtins::mb_float as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_bool",
            builtins::mb_bool as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_str",
            builtins::mb_str as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_abs",
            builtins::mb_abs as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_type",
            builtins::mb_type as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_type_no_args",
            builtins::mb_type_no_args as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_type2",
            builtins::mb_type2 as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_type3",
            builtins::mb_type3
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_type3_kwargs",
            builtins::mb_type3_kwargs
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_builtin_type_obj",
            builtins::mb_builtin_type_obj as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_range",
            builtins::mb_range as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_range_no_args",
            builtins::mb_range_no_args as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_range_2",
            builtins::mb_range_2 as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_range_3",
            builtins::mb_range_3
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_range_too_many_args",
            builtins::mb_range_too_many_args as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_slice",
            builtins::mb_slice
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_slice_no_args",
            builtins::mb_slice_no_args as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_breakpoint",
            builtins::mb_breakpoint as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_breakpoint_call",
            builtins::mb_breakpoint_call as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_memoryview",
            builtins::mb_memoryview as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_dunder_import",
            builtins::mb_dunder_import as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_add",
            builtins::mb_add as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_sub",
            builtins::mb_sub as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_mul",
            builtins::mb_mul as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_div",
            builtins::mb_div as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_mod",
            builtins::mb_mod as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_bitor",
            builtins::mb_bitor as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_bitand",
            builtins::mb_bitand as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_bitxor",
            builtins::mb_bitxor as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_lshift",
            builtins::mb_lshift as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_rshift",
            builtins::mb_rshift as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_neg",
            builtins::mb_neg as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_eq",
            builtins::mb_eq as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_match_bool_literal",
            builtins::mb_match_bool_literal as fn(super::MbValue, i64) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_lt",
            builtins::mb_lt as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_not",
            builtins::mb_not as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_min",
            builtins::mb_min as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_max",
            builtins::mb_max as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_sum",
            builtins::mb_sum as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_sorted",
            builtins::mb_sorted as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_repr",
            builtins::mb_repr as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_hash",
            builtins::mb_hash as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_id",
            builtins::mb_id as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_input",
            builtins::mb_input as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_chr",
            builtins::mb_chr as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_ord",
            builtins::mb_ord as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_hex",
            builtins::mb_hex as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_oct",
            builtins::mb_oct as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_bin",
            builtins::mb_bin as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_pow",
            builtins::mb_pow as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_floordiv",
            builtins::mb_floordiv as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_gt",
            builtins::mb_gt as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_le",
            builtins::mb_le as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_ge",
            builtins::mb_ge as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_ne",
            builtins::mb_ne as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_is_truthy",
            builtins::mb_is_truthy as fn(super::MbValue) -> i64,
            [I64],
            I64
        ),
        // ── Missing builtins (#420) ──
        rt_sym!(
            "mb_any",
            builtins::mb_any as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_all",
            builtins::mb_all as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_round",
            builtins::mb_round as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_divmod",
            builtins::mb_divmod as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_format",
            builtins::mb_format as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_callable",
            builtins::mb_callable as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_map",
            builtins::mb_map as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_map_n",
            iter::mb_map_n as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_filter",
            builtins::mb_filter as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_call_spread",
            builtins::mb_call_spread as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_call_spread_kwargs",
            builtins::mb_call_spread_kwargs
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_iadd",
            class::mb_iadd as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_isub",
            class::mb_isub as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_imul",
            class::mb_imul as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_iand",
            class::mb_iand as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_ior",
            class::mb_ior as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_ixor",
            class::mb_ixor as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_ipow",
            class::mb_ipow as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── String ops ──
        rt_sym!(
            "mb_str_concat",
            string_ops::mb_str_concat as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_upper",
            string_ops::mb_str_upper as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_str_lower",
            string_ops::mb_str_lower as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_str_split",
            string_ops::mb_str_split
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_replace",
            string_ops::mb_str_replace
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_splitlines",
            string_ops::mb_str_splitlines as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_expandtabs",
            string_ops::mb_str_expandtabs as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_partition",
            string_ops::mb_str_partition as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_rpartition",
            string_ops::mb_str_rpartition as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_removeprefix",
            string_ops::mb_str_removeprefix as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_removesuffix",
            string_ops::mb_str_removesuffix as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_format_value",
            string_ops::mb_format_value as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_repeat",
            string_ops::mb_str_repeat as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_getitem",
            string_ops::mb_str_getitem as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_slice",
            string_ops::mb_str_slice
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_strip",
            string_ops::mb_str_strip as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_lstrip",
            string_ops::mb_str_lstrip as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_rstrip",
            string_ops::mb_str_rstrip as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_find",
            string_ops::mb_str_find
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_index",
            string_ops::mb_str_index
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_count",
            string_ops::mb_str_count
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_startswith",
            string_ops::mb_str_startswith
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_endswith",
            string_ops::mb_str_endswith
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_join",
            string_ops::mb_str_join as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_isdigit",
            string_ops::mb_str_isdigit as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_str_isalpha",
            string_ops::mb_str_isalpha as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_str_capitalize",
            string_ops::mb_str_capitalize as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_str_title",
            string_ops::mb_str_title as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_str_contains",
            string_ops::mb_str_contains as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_eq",
            string_ops::mb_str_eq as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_lt",
            string_ops::mb_str_lt as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_str_format",
            string_ops::mb_str_format as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── List ops ──
        rt_sym!(
            "mb_list_new",
            list_ops::mb_list_new as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_list_new_with_capacity",
            list_ops::mb_list_new_with_capacity as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // Fixed-arity small-literal constructors — collapse `1 + N` FFI calls into one.
        rt_sym!(
            "mb_list_new_1",
            list_ops::mb_list_new_1 as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_list_new_2",
            list_ops::mb_list_new_2 as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_new_3",
            list_ops::mb_list_new_3
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_new_4",
            list_ops::mb_list_new_4
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_new_5",
            list_ops::mb_list_new_5
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_new_6",
            list_ops::mb_list_new_6
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_new_7",
            list_ops::mb_list_new_7
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_new_8",
            list_ops::mb_list_new_8
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64, I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_new_9",
            list_ops::mb_list_new_9
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64, I64, I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_new_10",
            list_ops::mb_list_new_10
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64, I64, I64, I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_from_iterable",
            list_ops::mb_list_from_iterable as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_list_append",
            list_ops::mb_list_append as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_list_append_unchecked",
            list_ops::mb_list_append_unchecked as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_list_getitem",
            list_ops::mb_list_getitem as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_setitem",
            list_ops::mb_list_setitem as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_list_len",
            list_ops::mb_list_len as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_is_sequence",
            list_ops::mb_is_sequence as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_seq_for_unpack",
            list_ops::mb_seq_for_unpack as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_seq_len_boxed",
            list_ops::mb_seq_len_boxed as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_seq_len",
            list_ops::mb_seq_len as fn(super::MbValue) -> i64,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_seq_getitem",
            list_ops::mb_seq_getitem as fn(super::MbValue, i64) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_seq_slice",
            list_ops::mb_seq_slice
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_pop",
            list_ops::mb_list_pop as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_list_contains",
            list_ops::mb_list_contains as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_sort",
            list_ops::mb_list_sort as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_list_insert",
            list_ops::mb_list_insert as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_list_pop_at",
            list_ops::mb_list_pop_at as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_remove",
            list_ops::mb_list_remove as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_list_extend",
            list_ops::mb_list_extend as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_list_clear",
            list_ops::mb_list_clear as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_list_reverse",
            list_ops::mb_list_reverse as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_list_copy",
            list_ops::mb_list_copy as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_list_index",
            list_ops::mb_list_index as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_count",
            list_ops::mb_list_count as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_slice",
            list_ops::mb_list_slice
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_concat",
            list_ops::mb_list_concat as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_args_concat",
            list_ops::mb_args_concat as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_eq",
            list_ops::mb_list_eq as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── Dict ops ──
        rt_sym!(
            "mb_dict_new",
            dict_ops::mb_dict_new as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_dict_from_pairs",
            dict_ops::mb_dict_from_pairs as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_dict_setitem",
            dict_ops::mb_dict_setitem as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_dict_getitem",
            dict_ops::mb_dict_getitem as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_dict_len",
            dict_ops::mb_dict_len as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_dict_contains",
            dict_ops::mb_dict_contains as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_is_mapping",
            dict_ops::mb_is_mapping as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_dict_pop",
            dict_ops::mb_dict_pop
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_dict_get",
            dict_ops::mb_dict_get
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_dict_setdefault",
            dict_ops::mb_dict_setdefault
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_dict_keys",
            dict_ops::mb_dict_keys as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_dict_values",
            dict_ops::mb_dict_values as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_dict_items",
            dict_ops::mb_dict_items as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_dict_keys_view",
            dict_ops::mb_dict_keys_view as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_dict_values_view",
            dict_ops::mb_dict_values_view as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_dict_items_view",
            dict_ops::mb_dict_items_view as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_dict_update",
            dict_ops::mb_dict_update as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_dict_clear",
            dict_ops::mb_dict_clear as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_dict_copy",
            dict_ops::mb_dict_copy as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_dict_delitem",
            dict_ops::mb_dict_delitem as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_dict_eq",
            dict_ops::mb_dict_eq as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_dict_merge",
            dict_ops::mb_dict_merge as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_dict_or",
            dict_ops::mb_dict_or as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_dict_ior",
            dict_ops::mb_dict_ior as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── Tuple ops ──
        rt_sym!(
            "mb_tuple_new",
            tuple_ops::mb_tuple_new as fn() -> super::MbValue,
            [],
            I64
        ),
        // Fixed-arity tuple constructors (#2128) — collapse MakeTuple to a
        // single FFI call, skipping the intermediate List + its gc_track.
        rt_sym!(
            "mb_tuple_new_1",
            tuple_ops::mb_tuple_new_1 as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_tuple_new_2",
            tuple_ops::mb_tuple_new_2 as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_tuple_new_3",
            tuple_ops::mb_tuple_new_3
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_tuple_new_4",
            tuple_ops::mb_tuple_new_4
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_tuple_new_5",
            tuple_ops::mb_tuple_new_5
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_tuple_new_6",
            tuple_ops::mb_tuple_new_6
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_tuple_new_7",
            tuple_ops::mb_tuple_new_7
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_tuple_new_8",
            tuple_ops::mb_tuple_new_8
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64, I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_tuple_len",
            tuple_ops::mb_tuple_len as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_tuple_getitem",
            tuple_ops::mb_tuple_getitem as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_tuple_contains",
            tuple_ops::mb_tuple_contains as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_to_tuple",
            tuple_ops::mb_list_to_tuple as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_star_args_to_tuple",
            tuple_ops::mb_star_args_to_tuple as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_tuple_from_iterable",
            tuple_ops::mb_tuple_from_iterable as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Exception ──
        rt_sym!(
            "mb_raise",
            exception::mb_raise as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_arg_bind_error",
            exception::mb_arg_bind_error as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_reraise",
            exception::mb_reraise as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_has_exception",
            exception::mb_has_exception as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_catch_exception",
            exception::mb_catch_exception as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_clear_exception",
            exception::mb_clear_exception as fn(),
            [],
            Void
        ),
        rt_sym!(
            "mb_push_handler",
            exception::mb_push_handler as fn(bool),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_pop_handler",
            exception::mb_pop_handler as fn(),
            [],
            Void
        ),
        rt_sym!(
            "mb_save_handled_exc",
            exception::mb_save_handled_exc as fn() -> i64,
            [],
            I64
        ),
        rt_sym!(
            "mb_restore_handled_exc",
            exception::mb_restore_handled_exc as fn(i64),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_exception_matches",
            exception::mb_exception_matches as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_exception_new",
            exception::mb_exception_new as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_exception_new_with_args",
            exception::mb_exception_new_with_args
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_exception_new_with_args_and_kwargs",
            exception::mb_exception_new_with_args_and_kwargs
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_name_error_with_name",
            exception::mb_name_error_with_name as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_raise_from",
            exception::mb_raise_from as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_raise_with_context",
            exception::mb_raise_with_context as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_raise_from_with_context",
            exception::mb_raise_from_with_context
                as fn(super::MbValue, super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64, I64],
            Void
        ),
        // ── PEP 695 runtime type parameters ──
        rt_sym!(
            "mb_pep695_typevar",
            pep695::mb_pep695_typevar
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_pep695_type_alias",
            pep695::mb_pep695_type_alias
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        // ── Class ──
        rt_sym!(
            "mb_getattr",
            class::mb_getattr as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_setattr",
            class::mb_setattr as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_isinstance",
            class::mb_isinstance as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_match_pos_arg",
            class::mb_match_pos_arg as fn(super::MbValue, super::MbValue, i64) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_class_set_match_args",
            class::mb_class_set_match_args as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_instance_hasattr",
            class::mb_instance_hasattr as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_class_has_pos_match",
            class::mb_class_has_pos_match
                as fn(super::MbValue, super::MbValue, i64) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_instance_new",
            class::mb_instance_new as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_instance_new_with_init",
            class::mb_instance_new_with_init
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_class_define",
            class::mb_class_define
                as fn(super::MbValue, super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_class_define_multi",
            class::mb_class_define_multi
                as fn(super::MbValue, super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_class_update_bases",
            class::mb_class_update_bases as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_class_set_metaclass",
            class::mb_class_set_metaclass as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_class_finalize_definition",
            class::mb_class_finalize_definition as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_class_set_namedtuple_base",
            class::mb_class_set_namedtuple_base
                as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_class_set_doc",
            class::mb_class_set_doc as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_class_set_abstractmethods",
            class::mb_class_set_abstractmethods as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_class_set_class_attr",
            class::mb_class_set_class_attr as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_class_set_kwargs",
            class::mb_class_set_kwargs as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        // PEP 557: ordered dataclass field facts, recorded right before the
        // @dataclass decorator call (see hir_to_mir ClassDefPlaceholder).
        rt_sym!(
            "mb_dataclass_record_field",
            super::stdlib::dataclasses_mod::mb_dataclass_record_field
                as fn(super::MbValue, super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_dataclass_record_field_nodefault",
            super::stdlib::dataclasses_mod::mb_dataclass_record_field_nodefault
                as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_raise_instance",
            class::mb_raise_instance as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_raise_instance_with_context",
            class::mb_raise_instance_with_context as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_raise_instance_from",
            class::mb_raise_instance_from as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_raise_instance_from_with_context",
            class::mb_raise_instance_from_with_context
                as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_catch_exception_instance",
            class::mb_catch_exception_instance as fn() -> super::MbValue,
            [],
            I64
        ),
        // ── Iterator ──
        rt_sym!(
            "mb_iter",
            iter::mb_iter as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_iter_sentinel",
            iter::mb_iter_sentinel as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_next",
            iter::mb_next as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_next_raise",
            iter::mb_next_raise as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_next_default",
            iter::mb_next_default as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_has_next",
            iter::mb_has_next as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_next_or_stop",
            iter::mb_next_or_stop as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_is_stop_iter",
            iter::mb_is_stop_iter as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_iter_release",
            iter::mb_iter_release as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_stop_iteration",
            iter::mb_stop_iteration as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_range_iter",
            iter::mb_range_iter
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_enumerate",
            iter::mb_enumerate as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_reversed",
            iter::mb_reversed as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_list_from_iter",
            iter::mb_list_from_iter as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Generator ──
        rt_sym!(
            "mb_generator_create",
            generator::mb_generator_create as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_generator_store_arg",
            generator::mb_generator_store_arg as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_generator_set_local_names",
            generator::mb_generator_set_local_names as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_generator_next",
            generator::mb_generator_next as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_generator_send",
            generator::mb_generator_send as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_generator_throw",
            generator::mb_generator_throw
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_generator_close",
            generator::mb_generator_close as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_generator_release",
            generator::mb_generator_release as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_del_var",
            generator::mb_del_var as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_generator_yield_value",
            generator::mb_generator_yield_value as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_generator_yield_from",
            generator::mb_generator_yield_from as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_generator_stop_value",
            generator::mb_generator_stop_value as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_generator_is_exhausted",
            generator::mb_generator_is_exhausted as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Closure ──
        rt_sym!(
            "mb_closure_new",
            closure::mb_closure_new
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_closure_get_capture",
            closure::mb_closure_get_capture as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_closure_set_capture",
            closure::mb_closure_set_capture as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_closure_release",
            closure::mb_closure_release as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_closure_set_defaults",
            closure::mb_closure_set_defaults as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_closure_set_arity",
            closure::mb_closure_set_arity as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_func_set_name",
            closure::mb_func_set_name as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_func_set_doc",
            closure::mb_func_set_doc as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_func_set_argcount",
            closure::mb_func_set_argcount as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_func_set_varnames",
            closure::mb_func_set_varnames as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_func_set_freevars",
            closure::mb_func_set_freevars as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_func_set_params",
            closure::mb_func_set_params as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_func_set_retanno",
            closure::mb_func_set_retanno as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_func_set_srcinfo",
            closure::mb_func_set_srcinfo as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_traceback_walk_stack_frame",
            traceback_mod::mb_traceback_walk_stack_frame
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_singledispatch_register_annotation",
            functools_mod::mb_singledispatch_register_annotation
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_functools_singledispatchmethod",
            functools_mod::mb_functools_singledispatchmethod
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_singledispatchmethod_register_value",
            functools_mod::mb_singledispatchmethod_register_value
                as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_fstring_value",
            string_ops::mb_fstring_value as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_apply_decorator",
            closure::mb_apply_decorator as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_global_get",
            closure::mb_global_get as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_global_set",
            closure::mb_global_set as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_global_get_id",
            closure::mb_global_get_id as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_global_set_id",
            closure::mb_global_set_id as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_global_del_id",
            closure::mb_global_del_id as fn(super::MbValue),
            [I64],
            Void
        ),
        // ── Cell variables (nonlocal) ──
        rt_sym!(
            "mb_cell_new",
            closure::mb_cell_new as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_cell_get",
            closure::mb_cell_get as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_cell_set",
            closure::mb_cell_set as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        // ── Module ──
        rt_sym!(
            "mb_import",
            module::mb_import as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_module_getattr",
            module::mb_module_getattr as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_builtin_get",
            module::mb_builtin_get as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_module_setattr",
            module::mb_module_setattr as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_import_from",
            module::mb_import_from as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // @spec .aw/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#R3
        rt_sym!(
            "mb_import_star",
            module::mb_import_star as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_add_search_path",
            module::mb_add_search_path as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_register_builtins",
            module::mb_register_builtins as fn(),
            [],
            Void
        ),
        // ── Async ──
        rt_sym!(
            "mb_coroutine_new",
            async_rt::mb_coroutine_new as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_await",
            async_rt::mb_await as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_coroutine_complete",
            async_rt::mb_coroutine_complete as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_coroutine_release",
            async_rt::mb_coroutine_release as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_coroutine_set_body",
            async_rt::mb_coroutine_set_body as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_coroutine_set_close_raises",
            async_rt::mb_coroutine_set_close_raises as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_coroutine_should_suspend",
            async_rt::mb_coroutine_should_suspend as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_coroutine_get_local",
            async_rt::mb_coroutine_get_local
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_coroutine_set_local",
            async_rt::mb_coroutine_set_local as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_create_task",
            async_rt::mb_create_task as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_cancel_task",
            async_rt::mb_cancel_task as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_task_done",
            async_rt::mb_task_done as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_task_result",
            async_rt::mb_task_result as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_gather",
            async_rt::mb_gather as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_sleep",
            async_rt::mb_sleep as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_run_until_complete",
            async_rt::mb_run_until_complete as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Async: Orbit bridge (#313 R2) ──
        rt_sym!(
            "mb_orbit_schedule",
            async_rt::mb_orbit_schedule as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_orbit_register_waker",
            async_rt::mb_orbit_register_waker as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Async: GIL (#313 R3) ──
        rt_sym!("mb_gil_release", async_rt::mb_gil_release as fn(), [], Void),
        rt_sym!("mb_gil_acquire", async_rt::mb_gil_acquire as fn(), [], Void),
        rt_sym!(
            "mb_gil_held",
            async_rt::mb_gil_held as fn() -> super::MbValue,
            [],
            I64
        ),
        // ── Async: Future interop (#313 R4) ──
        rt_sym!(
            "mb_await_external",
            async_rt::mb_await_external as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Async: Tokio multi-threaded executor (R6) ──
        rt_sym!(
            "mb_tokio_spawn",
            tokio_exec::mb_tokio_spawn as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_tokio_gather",
            tokio_exec::mb_tokio_gather as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Property / classmethod / staticmethod ──
        rt_sym!(
            "mb_property_new",
            class::mb_property_new as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_property_from_args",
            class::mb_property_from_args as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_property_setter",
            class::mb_property_setter as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_property_deleter",
            class::mb_property_deleter as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_property_get",
            class::mb_property_get as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_property_set",
            class::mb_property_set as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_classmethod_new",
            class::mb_classmethod_new as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_staticmethod_new",
            class::mb_staticmethod_new as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_cached_property_new",
            class::mb_cached_property_new as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_cached_property_get",
            class::mb_cached_property_get as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_descriptor_unwrap",
            class::mb_descriptor_unwrap as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Metaclasses / ABC ──
        rt_sym!(
            "mb_abstractmethod",
            class::mb_abstractmethod as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_register_abstract",
            class::mb_register_abstract as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_check_abstract",
            class::mb_check_abstract as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── super() ──
        rt_sym!(
            "mb_super",
            class::mb_super as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_super_getattr",
            class::mb_super_getattr as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_delattr",
            class::mb_delattr as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_hasattr",
            class::mb_hasattr as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_getattr_default",
            class::mb_getattr_default
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_vars",
            class::mb_vars as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_dir",
            class::mb_dir as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_dir_no_args",
            class::mb_dir_no_args as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_dir_arity_error",
            class::mb_dir_arity_error as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_check_setattr_dunder",
            class::mb_check_setattr_dunder as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_check_delattr_dunder",
            class::mb_check_delattr_dunder as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Dunder dispatch ──
        rt_sym!(
            "mb_dispatch_binop",
            class::mb_dispatch_binop as fn(i64, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_dispatch_unaryop",
            class::mb_dispatch_unaryop as fn(i64, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_obj_getitem",
            class::mb_obj_getitem as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_obj_setitem",
            class::mb_obj_setitem
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_obj_contains",
            class::mb_obj_contains as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_call_method1",
            class::mb_call_method1 as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_call0",
            class::mb_call0 as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_call1_val",
            class::mb_call1_val as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_lookup_dunder",
            class::mb_lookup_dunder as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_obj_len",
            class::mb_obj_len as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_obj_str",
            class::mb_obj_str as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_obj_repr",
            class::mb_obj_repr as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_obj_bool",
            class::mb_obj_bool as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_obj_hash",
            class::mb_obj_hash as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_issubclass",
            class::mb_issubclass as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_call_method",
            class::mb_call_method
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_call_method_kwargs",
            class::mb_call_method_kwargs
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64],
            I64
        ),
        // ── GC ──
        rt_sym!(
            "mb_gc_collect",
            super::gc::mb_gc_collect as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!("mb_gc_enable", super::gc::mb_gc_enable as fn(), [], Void),
        rt_sym!("mb_gc_disable", super::gc::mb_gc_disable as fn(), [], Void),
        rt_sym!(
            "mb_gc_isenabled",
            super::gc::mb_gc_isenabled as fn() -> super::MbValue,
            [],
            I64
        ),
        // ── Stdlib: sys ──
        rt_sym!(
            "mb_sys_exit",
            super::stdlib::sys_mod::mb_sys_exit as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_sys_getrecursionlimit",
            super::stdlib::sys_mod::mb_sys_getrecursionlimit as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_sys_getsizeof",
            super::stdlib::sys_mod::mb_sys_getsizeof as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_sys_getframe_with_locals",
            super::stdlib::sys_mod::mb_sys_getframe_with_locals
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: os ──
        rt_sym!(
            "mb_os_getcwd",
            super::stdlib::os_mod::mb_os_getcwd as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_os_listdir",
            super::stdlib::os_mod::mb_os_listdir as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_mkdir",
            super::stdlib::os_mod::mb_os_mkdir as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_remove",
            super::stdlib::os_mod::mb_os_remove as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_getenv",
            super::stdlib::os_mod::mb_os_getenv
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_os_path_join",
            super::stdlib::os_mod::mb_os_path_join
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_os_path_exists",
            super::stdlib::os_mod::mb_os_path_exists as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_path_isfile",
            super::stdlib::os_mod::mb_os_path_isfile as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_path_isdir",
            super::stdlib::os_mod::mb_os_path_isdir as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_path_basename",
            super::stdlib::os_mod::mb_os_path_basename as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_path_dirname",
            super::stdlib::os_mod::mb_os_path_dirname as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_path_abspath",
            super::stdlib::os_mod::mb_os_path_abspath as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_path_splitext",
            super::stdlib::os_mod::mb_os_path_splitext as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_path_split",
            super::stdlib::os_mod::mb_os_path_split as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_path_expanduser",
            super::stdlib::os_mod::mb_os_path_expanduser as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_path_getsize",
            super::stdlib::os_mod::mb_os_path_getsize as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_rename",
            super::stdlib::os_mod::mb_os_rename
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_os_makedirs",
            super::stdlib::os_mod::mb_os_makedirs as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_rmdir",
            super::stdlib::os_mod::mb_os_rmdir as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_os_walk",
            super::stdlib::os_mod::mb_os_walk as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: math ──
        rt_sym!(
            "mb_math_sqrt",
            super::stdlib::math_mod::mb_math_sqrt as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_math_floor",
            super::stdlib::math_mod::mb_math_floor as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_math_ceil",
            super::stdlib::math_mod::mb_math_ceil as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_math_sin",
            super::stdlib::math_mod::mb_math_sin as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_math_cos",
            super::stdlib::math_mod::mb_math_cos as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_math_pow",
            super::stdlib::math_mod::mb_math_pow
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_math_log",
            super::stdlib::math_mod::mb_math_log as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_math_factorial",
            super::stdlib::math_mod::mb_math_factorial as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_math_gcd",
            super::stdlib::math_mod::mb_math_gcd
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── Stdlib: time ──
        rt_sym!(
            "mb_time_time",
            super::stdlib::time_mod::mb_time_time as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_time_sleep",
            super::stdlib::time_mod::mb_time_sleep as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_time_monotonic",
            super::stdlib::time_mod::mb_time_monotonic as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_time_perf_counter",
            super::stdlib::time_mod::mb_time_perf_counter as fn() -> super::MbValue,
            [],
            I64
        ),
        // ── Stdlib: json ──
        rt_sym!(
            "mb_json_dumps",
            super::stdlib::json_mod::mb_json_dumps as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_json_loads",
            super::stdlib::json_mod::mb_json_loads as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── File I/O ──
        rt_sym!(
            "mb_open",
            file_io::mb_open as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_open_ex",
            file_io::mb_open_ex
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_open_kwargs",
            file_io::mb_open_kwargs
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_open_with_opener",
            file_io::mb_open_with_opener
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_file_read",
            file_io::mb_file_read as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_file_readline",
            file_io::mb_file_readline as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_file_readlines",
            file_io::mb_file_readlines as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_file_write",
            file_io::mb_file_write as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_file_close",
            file_io::mb_file_close as fn(super::MbValue),
            [I64],
            Void
        ),
        // ── Assert / Del ──
        rt_sym!(
            "mb_assertion_error",
            builtins::mb_assertion_error as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_assertion_error_no_msg",
            builtins::mb_assertion_error_no_msg as fn(),
            [],
            Void
        ),
        rt_sym!(
            "mb_list_delitem",
            list_ops::mb_list_delitem as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_obj_delitem",
            class::mb_obj_delitem as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        // ── Context Manager ──
        rt_sym!(
            "mb_context_enter",
            class::mb_context_enter as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_context_exit",
            class::mb_context_exit as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_async_context_enter",
            class::mb_async_context_enter as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_async_context_exit",
            class::mb_async_context_exit as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── Set ops (#386) ──
        rt_sym!(
            "mb_set_new",
            set_ops::mb_set_new as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_set_from_list",
            set_ops::mb_set_from_list as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_set_add",
            set_ops::mb_set_add as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_set_remove",
            set_ops::mb_set_remove as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_set_discard",
            set_ops::mb_set_discard as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_set_contains",
            set_ops::mb_set_contains as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_set_len",
            set_ops::mb_set_len as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_set_union",
            set_ops::mb_set_union as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_set_intersection",
            set_ops::mb_set_intersection as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_set_difference",
            set_ops::mb_set_difference as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_set_symmetric_difference",
            set_ops::mb_set_symmetric_difference
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_set_clear",
            set_ops::mb_set_clear as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_set_copy",
            set_ops::mb_set_copy as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_set_issubset",
            set_ops::mb_set_issubset as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_set_issuperset",
            set_ops::mb_set_issuperset as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_set_isdisjoint",
            set_ops::mb_set_isdisjoint as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── Bytes/ByteArray (#405) ──
        rt_sym!(
            "mb_bytes_new",
            super::bytes_ops::mb_bytes_new as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_bytes_new_checked",
            super::bytes_ops::mb_bytes_new_checked as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_bytes_new_encoded",
            super::bytes_ops::mb_bytes_new_encoded
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_bytearray_new",
            super::bytes_ops::mb_bytearray_new as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_bytearray_new_checked",
            super::bytes_ops::mb_bytearray_new_checked as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_bytearray_new_encoded",
            super::bytes_ops::mb_bytearray_new_encoded
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_bytes_getitem",
            super::bytes_ops::mb_bytes_getitem
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_bytes_len",
            super::bytes_ops::mb_bytes_len as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_bytes_decode",
            super::bytes_ops::mb_bytes_decode
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_bytes_hex",
            super::bytes_ops::mb_bytes_hex as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_bytes_find",
            super::bytes_ops::mb_bytes_find as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_bytes_concat",
            super::bytes_ops::mb_bytes_concat
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_bytes_contains",
            super::bytes_ops::mb_bytes_contains
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_bytearray_append",
            super::bytes_ops::mb_bytearray_append as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_bytearray_extend",
            super::bytes_ops::mb_bytearray_extend as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_bytearray_clear",
            super::bytes_ops::mb_bytearray_clear as fn(super::MbValue),
            [I64],
            Void
        ),
        rt_sym!(
            "mb_bytearray_pop",
            super::bytes_ops::mb_bytearray_pop as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_bytearray_reverse",
            super::bytes_ops::mb_bytearray_reverse as fn(super::MbValue),
            [I64],
            Void
        ),
        // ── Zip ──
        rt_sym!(
            "mb_zip",
            iter::mb_zip as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_zip_n",
            iter::mb_zip_n as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_zip_strict",
            iter::mb_zip_strict as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── ascii / sum_with_start (#R5) ──
        rt_sym!(
            "mb_ascii",
            builtins::mb_ascii as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_sum_with_start",
            builtins::mb_sum_with_start as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── Kwargs-aware builtins (xfail-reduction) ──
        rt_sym!(
            "mb_print_kwargs",
            builtins::mb_print_kwargs
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_print_kwargs_file",
            builtins::mb_print_kwargs_file
                as fn(
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                    super::MbValue,
                ) -> super::MbValue,
            [I64, I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_sorted_kwargs",
            builtins::mb_sorted_kwargs
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_min_kwargs",
            builtins::mb_min_kwargs
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_max_kwargs",
            builtins::mb_max_kwargs
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_pow_mod",
            builtins::mb_pow_mod
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_int_base",
            builtins::mb_int_base as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_list_sort_kwargs",
            list_ops::mb_list_sort_kwargs as fn(super::MbValue, super::MbValue, super::MbValue),
            [I64, I64, I64],
            Void
        ),
        rt_sym!(
            "mb_str_format_kwargs",
            string_ops::mb_str_format_kwargs
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        // ── __slots__, __format__, __del__ (#410) ──
        rt_sym!(
            "mb_register_slots",
            class::mb_register_slots as fn(super::MbValue, super::MbValue),
            [I64, I64],
            Void
        ),
        rt_sym!(
            "mb_obj_format",
            class::mb_obj_format as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_obj_del",
            class::mb_obj_del as fn(super::MbValue),
            [I64],
            Void
        ),
        // ── ExceptionGroup / except* (#410) ──
        rt_sym!(
            "mb_exception_group_new",
            exception::mb_exception_group_new
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_exception_group_construct",
            exception::mb_exception_group_construct
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_exception_group_construct_and_raise",
            exception::mb_exception_group_construct_and_raise
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_except_star",
            exception::mb_except_star as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_exception_group_split",
            exception::mb_exception_group_split
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_exception_group_subgroup",
            exception::mb_exception_group_subgroup
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_exception_group_exceptions",
            exception::mb_exception_group_exceptions as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Exception state retrieval (non-clearing) ──
        rt_sym!(
            "mb_get_exception",
            exception::mb_get_exception as fn() -> super::MbValue,
            [],
            I64
        ),
        // ── Exception class registration ──
        rt_sym!(
            "mb_register_builtin_exceptions",
            exception::register_builtin_exceptions as fn(),
            [],
            Void
        ),
        // ── FrozenSet / Set constructors (#410) ──
        rt_sym!(
            "mb_frozenset_new",
            builtins::mb_frozenset_new as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_frozenset_empty",
            builtins::mb_frozenset_empty as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_set_from_iterable",
            builtins::mb_set_from_iterable as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── eval/exec/compile/globals/locals (#441) ──
        rt_sym!(
            "mb_eval",
            builtins::mb_eval as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_exec",
            builtins::mb_exec as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_exec_with_globals",
            builtins::mb_exec_with_globals as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_compile",
            builtins::mb_compile
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_globals",
            builtins::mb_globals as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_locals",
            builtins::mb_locals as fn() -> super::MbValue,
            [],
            I64
        ),
        // ── Stdlib: subprocess (#397) ──
        rt_sym!(
            "mb_subprocess_run",
            super::stdlib::subprocess_mod::mb_subprocess_run
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_subprocess_call",
            super::stdlib::subprocess_mod::mb_subprocess_call
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_subprocess_check_output",
            super::stdlib::subprocess_mod::mb_subprocess_check_output
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_subprocess_check_call",
            super::stdlib::subprocess_mod::mb_subprocess_check_call
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: csv (#398) ──
        rt_sym!(
            "mb_csv_reader",
            super::stdlib::csv_mod::mb_csv_reader
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_csv_writer",
            super::stdlib::csv_mod::mb_csv_writer
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_csv_dictreader",
            super::stdlib::csv_mod::mb_csv_dictreader
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_csv_dictwriter",
            super::stdlib::csv_mod::mb_csv_dictwriter
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── Stdlib: argparse (#399) ──
        rt_sym!(
            "mb_argparse_new",
            super::stdlib::argparse_mod::mb_argparse_new as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_argparse_add_argument",
            super::stdlib::argparse_mod::mb_argparse_add_argument
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_argparse_parse_args",
            super::stdlib::argparse_mod::mb_argparse_parse_args
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: logging (#400) — dispatchers self-register via
        //    NATIVE_FUNC_ADDRS in logging_mod::register(); no rt_sym! needed. ──
        // ── Stdlib: typing (#401) ──
        rt_sym!(
            "mb_typing_cast",
            super::stdlib::typing_mod::mb_typing_cast
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_typing_get_type_hints",
            super::stdlib::typing_mod::mb_typing_get_type_hints
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: threading (#417) ──
        rt_sym!(
            "mb_threading_current_thread",
            super::stdlib::threading_mod::mb_threading_current_thread as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_threading_active_count",
            super::stdlib::threading_mod::mb_threading_active_count as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_threading_thread",
            super::stdlib::threading_mod::mb_threading_thread
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_threading_lock",
            super::stdlib::threading_mod::mb_threading_lock as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_threading_rlock",
            super::stdlib::threading_mod::mb_threading_rlock as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_threading_event",
            super::stdlib::threading_mod::mb_threading_event as fn() -> super::MbValue,
            [],
            I64
        ),
        // ── Stdlib: socket (#418) ──
        rt_sym!(
            "mb_socket_new",
            super::stdlib::socket_mod::mb_socket_new
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_socket_connect",
            super::stdlib::socket_mod::mb_socket_connect
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_socket_send",
            super::stdlib::socket_mod::mb_socket_send
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_socket_recv",
            super::stdlib::socket_mod::mb_socket_recv
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_socket_close",
            super::stdlib::socket_mod::mb_socket_close as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_socket_gethostname",
            super::stdlib::socket_mod::mb_socket_gethostname as fn() -> super::MbValue,
            [],
            I64
        ),
        // ── Stdlib: http/urllib (#418) ──
        rt_sym!(
            "mb_urllib_urlopen",
            super::stdlib::http_mod::mb_urllib_urlopen as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_urllib_urlencode",
            super::stdlib::http_mod::mb_urllib_urlencode as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_urllib_quote",
            super::stdlib::http_mod::mb_urllib_quote
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_urllib_unquote",
            super::stdlib::http_mod::mb_urllib_unquote as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_urllib_urlparse",
            super::stdlib::http_mod::mb_urllib_urlparse as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: unittest (#419) ──
        rt_sym!(
            "mb_unittest_testcase",
            super::stdlib::unittest_mod::mb_unittest_testcase as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_unittest_main",
            super::stdlib::unittest_mod::mb_unittest_main as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_unittest_assert_equal",
            super::stdlib::unittest_mod::mb_unittest_assert_equal
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_unittest_assert_true",
            super::stdlib::unittest_mod::mb_unittest_assert_true
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_unittest_assert_false",
            super::stdlib::unittest_mod::mb_unittest_assert_false
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_unittest_assert_is_none",
            super::stdlib::unittest_mod::mb_unittest_assert_is_none
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: pickle (#442) ──
        rt_sym!(
            "mb_pickle_dumps",
            super::stdlib::pickle_mod::mb_pickle_dumps as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_pickle_loads",
            super::stdlib::pickle_mod::mb_pickle_loads as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: sqlite3 (#444) ──
        rt_sym!(
            "mb_sqlite3_connect",
            super::stdlib::sqlite3_mod::mb_sqlite3_connect as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: gzip (#445) ──
        rt_sym!(
            "mb_gzip_compress",
            super::stdlib::gzip_mod::mb_gzip_compress as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_gzip_decompress",
            super::stdlib::gzip_mod::mb_gzip_decompress as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: pprint (#446) ──
        rt_sym!(
            "mb_pprint_pformat",
            super::stdlib::pprint_mod::mb_pprint_pformat as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_pprint_pprint",
            super::stdlib::pprint_mod::mb_pprint_pprint as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: textwrap (#448) ──
        rt_sym!(
            "mb_textwrap_wrap",
            super::stdlib::textwrap_mod::mb_textwrap_wrap
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_textwrap_fill",
            super::stdlib::textwrap_mod::mb_textwrap_fill
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_textwrap_dedent",
            super::stdlib::textwrap_mod::mb_textwrap_dedent as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_textwrap_indent",
            super::stdlib::textwrap_mod::mb_textwrap_indent
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_textwrap_shorten",
            super::stdlib::textwrap_mod::mb_textwrap_shorten
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── Stdlib: string (#452) ──
        rt_sym!(
            "mb_string_capwords",
            super::stdlib::string_constants_mod::mb_string_capwords
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── Stdlib: cmath (#453) ── registered via tuple-table + NATIVE_FUNC_ADDRS in cmath_mod.rs (#1265 Task #38)
        // ── Stdlib: array (#451) ──
        rt_sym!(
            "mb_array_new",
            super::stdlib::array_mod::mb_array_new
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_array_append",
            super::stdlib::array_mod::mb_array_append
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_array_tolist",
            super::stdlib::array_mod::mb_array_tolist as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: xml (#449) ──
        rt_sym!(
            "mb_xml_element",
            super::stdlib::xml_mod::mb_xml_element
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_xml_subelement",
            super::stdlib::xml_mod::mb_xml_subelement
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_xml_tostring",
            super::stdlib::xml_mod::mb_xml_tostring as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: html (#449) ──
        rt_sym!(
            "mb_html_escape",
            super::stdlib::html_parser_mod::mb_html_escape as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_html_unescape",
            super::stdlib::html_parser_mod::mb_html_unescape
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── BigInt overflow fallback (#833) ──
        RuntimeSymbol {
            name: "mb_bigint_add",
            addr: super::bigint_ops::mb_bigint_add as *const u8,
            params: &[I64, I64],
            return_type: I64,
        },
        RuntimeSymbol {
            name: "mb_bigint_sub",
            addr: super::bigint_ops::mb_bigint_sub as *const u8,
            params: &[I64, I64],
            return_type: I64,
        },
        RuntimeSymbol {
            name: "mb_bigint_mul",
            addr: super::bigint_ops::mb_bigint_mul as *const u8,
            params: &[I64, I64],
            return_type: I64,
        },
        RuntimeSymbol {
            name: "mb_bigint_cmp",
            addr: super::bigint_ops::mb_bigint_cmp as *const u8,
            params: &[I64, I64],
            return_type: I64,
        },
        RuntimeSymbol {
            name: "mb_bigint_eq",
            addr: super::bigint_ops::mb_bigint_eq as *const u8,
            params: &[I64, I64],
            return_type: I64,
        },
        RuntimeSymbol {
            name: "mb_bigint_hash",
            addr: super::bigint_ops::mb_bigint_hash as *const u8,
            params: &[I64],
            return_type: I64,
        },
        RuntimeSymbol {
            name: "mb_bigint_retain",
            addr: super::bigint_ops::mb_bigint_retain as *const u8,
            params: &[I64],
            return_type: Void,
        },
        RuntimeSymbol {
            name: "mb_bigint_release",
            addr: super::bigint_ops::mb_bigint_release as *const u8,
            params: &[I64],
            return_type: Void,
        },
        RuntimeSymbol {
            name: "mb_bigint_from_i64",
            addr: super::bigint_ops::mb_bigint_from_i64 as *const u8,
            params: &[I64],
            return_type: I64,
        },
        // ── Complex number support (R3 CPython 3.12 conformance) ──
        rt_sym!(
            "mb_complex",
            builtins::mb_complex as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── Stdlib: keyword (#690) ──
        rt_sym!(
            "mb_keyword_kwlist",
            super::stdlib::keyword_mod::mb_keyword_kwlist as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_keyword_iskeyword",
            super::stdlib::keyword_mod::mb_keyword_iskeyword
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_keyword_softkwlist",
            super::stdlib::keyword_mod::mb_keyword_softkwlist as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_keyword_issoftkeyword",
            super::stdlib::keyword_mod::mb_keyword_issoftkeyword
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: token (#698) ──
        rt_sym!(
            "mb_token_tok_name",
            super::stdlib::token_mod::mb_token_tok_name as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_token_exact_token_types",
            super::stdlib::token_mod::mb_token_exact_token_types as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_token_isterminal",
            super::stdlib::token_mod::mb_token_isterminal as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_token_isnonterminal",
            super::stdlib::token_mod::mb_token_isnonterminal
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_token_iseof",
            super::stdlib::token_mod::mb_token_iseof as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: stat ──
        rt_sym!(
            "mb_stat_s_isdir",
            super::stdlib::stat_mod::mb_stat_s_isdir as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_stat_s_ischr",
            super::stdlib::stat_mod::mb_stat_s_ischr as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_stat_s_isblk",
            super::stdlib::stat_mod::mb_stat_s_isblk as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_stat_s_isreg",
            super::stdlib::stat_mod::mb_stat_s_isreg as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_stat_s_isfifo",
            super::stdlib::stat_mod::mb_stat_s_isfifo as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_stat_s_islnk",
            super::stdlib::stat_mod::mb_stat_s_islnk as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_stat_s_issock",
            super::stdlib::stat_mod::mb_stat_s_issock as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_stat_s_imode",
            super::stdlib::stat_mod::mb_stat_s_imode as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_stat_s_ifmt_fn",
            super::stdlib::stat_mod::mb_stat_s_ifmt_fn as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_stat_filemode",
            super::stdlib::stat_mod::mb_stat_filemode as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: fnmatch (#670) ──
        rt_sym!(
            "mb_fnmatch_fnmatch",
            super::stdlib::fnmatch_mod::mb_fnmatch_fnmatch
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_fnmatch_fnmatchcase",
            super::stdlib::fnmatch_mod::mb_fnmatch_fnmatchcase
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_fnmatch_filter",
            super::stdlib::fnmatch_mod::mb_fnmatch_filter
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_fnmatch_translate",
            super::stdlib::fnmatch_mod::mb_fnmatch_translate
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: getopt ──
        rt_sym!(
            "mb_getopt_getopt",
            super::stdlib::getopt_mod::mb_getopt_getopt
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_getopt_gnu_getopt",
            super::stdlib::getopt_mod::mb_getopt_gnu_getopt
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        // ── Stdlib: graphlib ──
        rt_sym!(
            "mb_graphlib_new",
            super::stdlib::graphlib_mod::mb_graphlib_new as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_graphlib_add",
            super::stdlib::graphlib_mod::mb_graphlib_add
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_graphlib_prepare",
            super::stdlib::graphlib_mod::mb_graphlib_prepare
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_graphlib_get_ready",
            super::stdlib::graphlib_mod::mb_graphlib_get_ready
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_graphlib_done",
            super::stdlib::graphlib_mod::mb_graphlib_done
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_graphlib_is_active",
            super::stdlib::graphlib_mod::mb_graphlib_is_active
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_graphlib_static_order",
            super::stdlib::graphlib_mod::mb_graphlib_static_order
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        // ── Stdlib: linecache ──
        rt_sym!(
            "mb_linecache_getline",
            super::stdlib::linecache_mod::mb_linecache_getline
                as fn(super::MbValue, super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64, I64],
            I64
        ),
        rt_sym!(
            "mb_linecache_getlines",
            super::stdlib::linecache_mod::mb_linecache_getlines
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_linecache_clearcache",
            super::stdlib::linecache_mod::mb_linecache_clearcache as fn() -> super::MbValue,
            [],
            I64
        ),
        rt_sym!(
            "mb_linecache_checkcache",
            super::stdlib::linecache_mod::mb_linecache_checkcache
                as fn(super::MbValue) -> super::MbValue,
            [I64],
            I64
        ),
        rt_sym!(
            "mb_linecache_lazycache",
            super::stdlib::linecache_mod::mb_linecache_lazycache
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        rt_sym!(
            "mb_linecache_updatecache",
            super::stdlib::linecache_mod::mb_linecache_updatecache
                as fn(super::MbValue, super::MbValue) -> super::MbValue,
            [I64, I64],
            I64
        ),
        // ── Refcount JIT wrappers (#1129) ──
        RuntimeSymbol {
            name: "mb_retain_value",
            addr: super::rc::mb_retain_value as *const u8,
            params: &[I64],
            return_type: Void,
        },
        RuntimeSymbol {
            name: "mb_release_value",
            addr: super::rc::mb_release_value as *const u8,
            params: &[I64],
            return_type: Void,
        },
    ]
}

/// Convert runtime symbols to MirExtern declarations for codegen.
pub fn runtime_externs() -> Vec<MirExtern> {
    runtime_symbols()
        .into_iter()
        .map(|sym| MirExtern {
            name: sym.name.to_string(),
            params: sym.params.to_vec(),
            return_type: sym.return_type,
            lib_name: "mamba_rt".to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // Lock three invariants the codegen backend relies on:
    //   (1) every rt_sym! name starts with `mb_` (project-wide runtime convention);
    //   (2) no duplicate registrations (would silently shadow in JIT linking);
    //   (3) runtime_externs() mirrors runtime_symbols() 1:1 with lib_name=mamba_rt
    //       in the same order — codegen indexes by position elsewhere.
    // Also anchors the coroutine runtime symbols the JIT actually emits calls to
    // (mb_coroutine_new + mb_coroutine_complete — see hir_to_mir.rs:897 and :856)
    // against the spec-R2 drift found in runtime/async.md tick 155 (spec named
    // nonexistent mb_coroutine_suspend/resume). Note: mb_coroutine_step is called
    // from the Rust-side tokio executor, not from JIT-emitted code, so it lives
    // in async_rt but not in this registry.
    #[test]
    fn runtime_symbols_registry_invariants() {
        let syms = runtime_symbols();
        assert!(
            !syms.is_empty(),
            "runtime_symbols() must register at least one symbol"
        );
        let mut seen: HashSet<&'static str> = HashSet::new();
        for s in &syms {
            assert!(
                s.name.starts_with("mb_"),
                "symbol name must start with mb_: {}",
                s.name
            );
            assert!(
                seen.insert(s.name),
                "duplicate rt_sym! registration for {}",
                s.name
            );
        }
        let externs = runtime_externs();
        assert_eq!(
            externs.len(),
            syms.len(),
            "runtime_externs len must mirror runtime_symbols len"
        );
        for (s, e) in syms.iter().zip(externs.iter()) {
            assert_eq!(
                s.name, e.name,
                "extern name must match symbol name at same position"
            );
            assert_eq!(
                e.lib_name, "mamba_rt",
                "extern {} must declare lib_name=mamba_rt",
                e.name
            );
        }
        for expected in ["mb_coroutine_new", "mb_coroutine_complete"] {
            assert!(
                seen.contains(expected),
                "coroutine runtime symbol {} missing",
                expected
            );
        }
    }

    // Spot-check that critical cross-subsystem runtime symbols the JIT actually
    // emits CallExtern for are registered. Complements the structural
    // invariants test above by anchoring specific API-surface members against
    // silent deletions: mb_type3 (builtins.rs:631, tick-163 archival audit —
    // issue #974 type 3-arg class creation), mb_cell_* (closure.rs, backs
    // nonlocal variables per resolve/name-resolution spec), mb_import +
    // mb_module_getattr + mb_import_star (module.rs, backs #1132 import
    // lowering at hir_to_mir.rs:2066), mb_bigint_add (bigint_ops.rs, anchors
    // the bigint.md tick-159 drift finding — TAG_BIGINT name is wrong but the
    // symbol IS registered under mb_bigint_* prefix).
    // Anchor R2 typed-signature architecture: symbols that take multiple
    // params must declare them (not empty &[]). Catches accidental regression
    // to count-based or missing param declarations per symbols.md tick-217 drift.
    #[test]
    fn runtime_symbols_typed_params_not_empty() {
        let syms = runtime_symbols();
        let by_name: std::collections::HashMap<&str, &RuntimeSymbol> =
            syms.iter().map(|s| (s.name, s)).collect();
        // Known multi-param functions — if their params become &[] codegen breaks
        for (name, min_params) in [
            ("mb_str_split", 2),
            ("mb_list_append", 2),
            ("mb_dict_setitem", 3),
            ("mb_graphlib_add", 3),
        ] {
            let sym = by_name
                .get(name)
                .unwrap_or_else(|| panic!("{} missing", name));
            assert!(
                sym.params.len() >= min_params,
                "{} must declare >= {} params, got {}",
                name,
                min_params,
                sym.params.len()
            );
        }
    }

    #[test]
    fn runtime_symbols_critical_api_surface_present() {
        let syms = runtime_symbols();
        let seen: HashSet<&'static str> = syms.iter().map(|s| s.name).collect();
        for expected in [
            "mb_type3",
            "mb_cell_new",
            "mb_cell_get",
            "mb_cell_set",
            "mb_import",
            "mb_module_getattr",
            "mb_import_star",
            "mb_bigint_add",
        ] {
            assert!(
                seen.contains(expected),
                "critical runtime symbol {} missing from registry",
                expected
            );
        }
    }
}
