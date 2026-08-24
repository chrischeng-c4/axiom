use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_sitebuiltins/Quitter____call____code_as__ExitCode_wrong.py`.
#[test]
fn test_gen_type_std_libs__sitebuiltins_Quitter____call____code_as__ExitCode_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_sitebuiltins"
# dimension = "type"
# case = "Quitter____call____code_as__ExitCode_wrong"
# subject = "_sitebuiltins.Quitter.__call__(code: _ExitCode)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_sitebuiltins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _sitebuiltins.Quitter.__call__(code: _ExitCode); call it with the wrong type.

typeshed contract: code is _ExitCode. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _sitebuiltins import Quitter
obj = object.__new__(Quitter)
try:
    obj.__call__(_W())  # code: _ExitCode <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_sitebuiltins/Quitter__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__sitebuiltins_Quitter__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_sitebuiltins"
# dimension = "type"
# case = "Quitter__init__name_as_str_wrong"
# subject = "_sitebuiltins.Quitter.__init__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_sitebuiltins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _sitebuiltins.Quitter.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _sitebuiltins import Quitter
try:
    Quitter(12345, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
