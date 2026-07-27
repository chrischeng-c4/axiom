use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/numbers/Integral____round____ndigits_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_numbers_Integral____round____ndigits_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "type"
# case = "Integral____round____ndigits_as_int_wrong"
# subject = "numbers.Integral.__round__(ndigits: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/numbers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: numbers.Integral.__round__(ndigits: int); call it with the wrong type.

typeshed contract: ndigits is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from numbers import Integral
obj = object.__new__(Integral)
try:
    obj.__round__("not_an_int")  # ndigits: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/numbers/Integral____round____ndigits_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_numbers_Integral____round____ndigits_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "type"
# case = "Integral____round____ndigits_as_typed_wrong"
# subject = "numbers.Integral.__round__(ndigits: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/numbers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: numbers.Integral.__round__(ndigits: typed); call it with the wrong type.

typeshed contract: ndigits is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from numbers import Integral
obj = object.__new__(Integral)
try:
    obj.__round__(_W())  # ndigits: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/numbers/Real____round____ndigits_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_numbers_Real____round____ndigits_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "type"
# case = "Real____round____ndigits_as_int_wrong"
# subject = "numbers.Real.__round__(ndigits: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/numbers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: numbers.Real.__round__(ndigits: int); call it with the wrong type.

typeshed contract: ndigits is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from numbers import Real
obj = object.__new__(Real)
try:
    obj.__round__("not_an_int")  # ndigits: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/numbers/Real____round____ndigits_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_numbers_Real____round____ndigits_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "type"
# case = "Real____round____ndigits_as_typed_wrong"
# subject = "numbers.Real.__round__(ndigits: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/numbers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: numbers.Real.__round__(ndigits: typed); call it with the wrong type.

typeshed contract: ndigits is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from numbers import Real
obj = object.__new__(Real)
try:
    obj.__round__(_W())  # ndigits: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
