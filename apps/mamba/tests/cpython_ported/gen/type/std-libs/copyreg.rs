use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/copyreg/constructor__object_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_copyreg_constructor__object_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copyreg"
# dimension = "type"
# case = "constructor__object_as_Callable_wrong"
# subject = "copyreg.constructor(object: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/copyreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: copyreg.constructor(object: Callable); call it with the wrong type.

typeshed contract: object is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from copyreg import constructor
try:
    constructor(_W())  # object: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/copyreg/pickle__ob_type_as_type_wrong.py`.
#[test]
fn test_gen_type_std_libs_copyreg_pickle__ob_type_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copyreg"
# dimension = "type"
# case = "pickle__ob_type_as_type_wrong"
# subject = "copyreg.pickle(ob_type: type)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/copyreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: copyreg.pickle(ob_type: type); call it with the wrong type.

typeshed contract: ob_type is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from copyreg import pickle
try:
    pickle(_W(), None)  # ob_type: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
