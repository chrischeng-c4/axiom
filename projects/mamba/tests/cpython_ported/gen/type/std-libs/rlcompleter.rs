use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/rlcompleter/Completer__attr_matches__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_rlcompleter_Completer__attr_matches__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "rlcompleter"
# dimension = "type"
# case = "Completer__attr_matches__text_as_str_wrong"
# subject = "rlcompleter.Completer.attr_matches(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/rlcompleter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: rlcompleter.Completer.attr_matches(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from rlcompleter import Completer
obj = object.__new__(Completer)
try:
    obj.attr_matches(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/rlcompleter/Completer__complete__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_rlcompleter_Completer__complete__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "rlcompleter"
# dimension = "type"
# case = "Completer__complete__text_as_str_wrong"
# subject = "rlcompleter.Completer.complete(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/rlcompleter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: rlcompleter.Completer.complete(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from rlcompleter import Completer
obj = object.__new__(Completer)
try:
    obj.complete(12345, 0)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/rlcompleter/Completer__global_matches__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_rlcompleter_Completer__global_matches__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "rlcompleter"
# dimension = "type"
# case = "Completer__global_matches__text_as_str_wrong"
# subject = "rlcompleter.Completer.global_matches(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/rlcompleter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: rlcompleter.Completer.global_matches(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from rlcompleter import Completer
obj = object.__new__(Completer)
try:
    obj.global_matches(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/rlcompleter/Completer__init__namespace_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_rlcompleter_Completer__init__namespace_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "rlcompleter"
# dimension = "type"
# case = "Completer__init__namespace_as_typed_wrong"
# subject = "rlcompleter.Completer.__init__(namespace: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/rlcompleter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: rlcompleter.Completer.__init__(namespace: typed); call it with the wrong type.

typeshed contract: namespace is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from rlcompleter import Completer
try:
    Completer(_W())  # namespace: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
