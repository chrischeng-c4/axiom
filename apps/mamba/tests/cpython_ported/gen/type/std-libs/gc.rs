use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/gc/collect__generation_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_gc_collect__generation_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "type"
# case = "collect__generation_as_int_wrong"
# subject = "gc.collect(generation: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/gc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: gc.collect(generation: int); call it with the wrong type.

typeshed contract: generation is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from gc import collect
try:
    collect("not_an_int")  # generation: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/gc/get_objects__generation_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_gc_get_objects__generation_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "type"
# case = "get_objects__generation_as_typed_wrong"
# subject = "gc.get_objects(generation: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/gc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: gc.get_objects(generation: typed); call it with the wrong type.

typeshed contract: generation is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from gc import get_objects
try:
    get_objects(_W())  # generation: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/gc/set_debug__flags_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_gc_set_debug__flags_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "type"
# case = "set_debug__flags_as_int_wrong"
# subject = "gc.set_debug(flags: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/gc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: gc.set_debug(flags: int); call it with the wrong type.

typeshed contract: flags is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from gc import set_debug
try:
    set_debug("not_an_int")  # flags: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/gc/set_threshold__threshold0_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_gc_set_threshold__threshold0_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "type"
# case = "set_threshold__threshold0_as_int_wrong"
# subject = "gc.set_threshold(threshold0: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/gc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: gc.set_threshold(threshold0: int); call it with the wrong type.

typeshed contract: threshold0 is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from gc import set_threshold
try:
    set_threshold("not_an_int")  # threshold0: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
