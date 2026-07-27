use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/pstats/Stats__dump_stats__filename_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pstats_Stats__dump_stats__filename_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "type"
# case = "Stats__dump_stats__filename_as_StrOrBytesPath_wrong"
# subject = "pstats.Stats.dump_stats(filename: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pstats.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pstats.Stats.dump_stats(filename: StrOrBytesPath); call it with the wrong type.

typeshed contract: filename is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pstats import Stats
obj = object.__new__(Stats)
try:
    obj.dump_stats(_W())  # filename: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pstats/Stats__eval_print_amount__sel_as__Selector_wrong.py`.
#[test]
fn test_gen_type_std_libs_pstats_Stats__eval_print_amount__sel_as__Selector_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "type"
# case = "Stats__eval_print_amount__sel_as__Selector_wrong"
# subject = "pstats.Stats.eval_print_amount(sel: _Selector)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pstats.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pstats.Stats.eval_print_amount(sel: _Selector); call it with the wrong type.

typeshed contract: sel is _Selector. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pstats import Stats
obj = object.__new__(Stats)
try:
    obj.eval_print_amount(_W(), None, "")  # sel: _Selector <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pstats/Stats__get_print_list__sel_list_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_pstats_Stats__get_print_list__sel_list_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "type"
# case = "Stats__get_print_list__sel_list_as_Iterable_wrong"
# subject = "pstats.Stats.get_print_list(sel_list: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pstats.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pstats.Stats.get_print_list(sel_list: Iterable); call it with the wrong type.

typeshed contract: sel_list is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pstats import Stats
obj = object.__new__(Stats)
try:
    obj.get_print_list(_W())  # sel_list: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pstats/Stats__load_stats__arg_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_pstats_Stats__load_stats__arg_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "type"
# case = "Stats__load_stats__arg_as_typed_wrong"
# subject = "pstats.Stats.load_stats(arg: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pstats.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pstats.Stats.load_stats(arg: typed); call it with the wrong type.

typeshed contract: arg is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pstats import Stats
obj = object.__new__(Stats)
try:
    obj.load_stats(_W())  # arg: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pstats/Stats__print_call_heading__name_size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_pstats_Stats__print_call_heading__name_size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "type"
# case = "Stats__print_call_heading__name_size_as_int_wrong"
# subject = "pstats.Stats.print_call_heading(name_size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pstats.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pstats.Stats.print_call_heading(name_size: int); call it with the wrong type.

typeshed contract: name_size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pstats import Stats
obj = object.__new__(Stats)
try:
    obj.print_call_heading("not_an_int", "")  # name_size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pstats/Stats__print_call_line__name_size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_pstats_Stats__print_call_line__name_size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "type"
# case = "Stats__print_call_line__name_size_as_int_wrong"
# subject = "pstats.Stats.print_call_line(name_size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pstats.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pstats.Stats.print_call_line(name_size: int); call it with the wrong type.

typeshed contract: name_size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pstats import Stats
obj = object.__new__(Stats)
try:
    obj.print_call_line("not_an_int", "", None)  # name_size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pstats/Stats__print_call_subheading__name_size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_pstats_Stats__print_call_subheading__name_size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "type"
# case = "Stats__print_call_subheading__name_size_as_int_wrong"
# subject = "pstats.Stats.print_call_subheading(name_size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pstats.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pstats.Stats.print_call_subheading(name_size: int); call it with the wrong type.

typeshed contract: name_size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pstats import Stats
obj = object.__new__(Stats)
try:
    obj.print_call_subheading("not_an_int")  # name_size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pstats/Stats__print_line__func_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pstats_Stats__print_line__func_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "type"
# case = "Stats__print_line__func_as_str_wrong"
# subject = "pstats.Stats.print_line(func: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pstats.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pstats.Stats.print_line(func: str); call it with the wrong type.

typeshed contract: func is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pstats import Stats
obj = object.__new__(Stats)
try:
    obj.print_line(12345)  # func: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
