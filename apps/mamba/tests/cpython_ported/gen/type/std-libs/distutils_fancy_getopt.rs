use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/distutils_fancy_getopt/FancyGetopt__generate_help__header_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_fancy_getopt_FancyGetopt__generate_help__header_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_fancy_getopt"
# dimension = "type"
# case = "FancyGetopt__generate_help__header_as_typed_wrong"
# subject = "distutils.fancy_getopt.FancyGetopt.generate_help(header: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/fancy_getopt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.fancy_getopt.FancyGetopt.generate_help(header: typed); call it with the wrong type.

typeshed contract: header is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.fancy_getopt import FancyGetopt
obj = object.__new__(FancyGetopt)
try:
    obj.generate_help(_W())  # header: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_fancy_getopt/FancyGetopt__getopt__args_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_fancy_getopt_FancyGetopt__getopt__args_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_fancy_getopt"
# dimension = "type"
# case = "FancyGetopt__getopt__args_as_typed_wrong"
# subject = "distutils.fancy_getopt.FancyGetopt.getopt(args: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/fancy_getopt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.fancy_getopt.FancyGetopt.getopt(args: typed); call it with the wrong type.

typeshed contract: args is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.fancy_getopt import FancyGetopt
obj = object.__new__(FancyGetopt)
try:
    obj.getopt(_W())  # args: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_fancy_getopt/FancyGetopt__init__option_table_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_fancy_getopt_FancyGetopt__init__option_table_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_fancy_getopt"
# dimension = "type"
# case = "FancyGetopt__init__option_table_as_typed_wrong"
# subject = "distutils.fancy_getopt.FancyGetopt.__init__(option_table: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/fancy_getopt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.fancy_getopt.FancyGetopt.__init__(option_table: typed); call it with the wrong type.

typeshed contract: option_table is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.fancy_getopt import FancyGetopt
try:
    FancyGetopt(_W())  # option_table: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_fancy_getopt/OptionDummy__init__options_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_fancy_getopt_OptionDummy__init__options_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_fancy_getopt"
# dimension = "type"
# case = "OptionDummy__init__options_as_Iterable_wrong"
# subject = "distutils.fancy_getopt.OptionDummy.__init__(options: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/fancy_getopt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.fancy_getopt.OptionDummy.__init__(options: Iterable); call it with the wrong type.

typeshed contract: options is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.fancy_getopt import OptionDummy
try:
    OptionDummy(_W())  # options: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_fancy_getopt/fancy_getopt__options_as_list_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_fancy_getopt_fancy_getopt__options_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_fancy_getopt"
# dimension = "type"
# case = "fancy_getopt__options_as_list_wrong"
# subject = "distutils.fancy_getopt.fancy_getopt(options: list)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/fancy_getopt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.fancy_getopt.fancy_getopt(options: list); call it with the wrong type.

typeshed contract: options is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.fancy_getopt import fancy_getopt
try:
    fancy_getopt(12345, None, None, None)  # options: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_fancy_getopt/translate_longopt__opt_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_fancy_getopt_translate_longopt__opt_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_fancy_getopt"
# dimension = "type"
# case = "translate_longopt__opt_as_str_wrong"
# subject = "distutils.fancy_getopt.translate_longopt(opt: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/fancy_getopt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.fancy_getopt.translate_longopt(opt: str); call it with the wrong type.

typeshed contract: opt is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.fancy_getopt import translate_longopt
try:
    translate_longopt(12345)  # opt: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_fancy_getopt/wrap_text__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_fancy_getopt_wrap_text__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_fancy_getopt"
# dimension = "type"
# case = "wrap_text__text_as_str_wrong"
# subject = "distutils.fancy_getopt.wrap_text(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/fancy_getopt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.fancy_getopt.wrap_text(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.fancy_getopt import wrap_text
try:
    wrap_text(12345, 0)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
