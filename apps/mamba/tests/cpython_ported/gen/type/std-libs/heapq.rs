use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_heapq/heapify__heap_as_list_wrong.py`.
#[test]
fn test_gen_type_std_libs__heapq_heapify__heap_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_heapq"
# dimension = "type"
# case = "heapify__heap_as_list_wrong"
# subject = "_heapq.heapify(heap: list)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_heapq.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _heapq.heapify(heap: list); call it with the wrong type.

typeshed contract: heap is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _heapq import heapify
try:
    heapify(12345)  # heap: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_heapq/heappop__heap_as_list_wrong.py`.
#[test]
fn test_gen_type_std_libs__heapq_heappop__heap_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_heapq"
# dimension = "type"
# case = "heappop__heap_as_list_wrong"
# subject = "_heapq.heappop(heap: list)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_heapq.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _heapq.heappop(heap: list); call it with the wrong type.

typeshed contract: heap is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _heapq import heappop
try:
    heappop(12345)  # heap: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_heapq/heappush__heap_as_list_wrong.py`.
#[test]
fn test_gen_type_std_libs__heapq_heappush__heap_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_heapq"
# dimension = "type"
# case = "heappush__heap_as_list_wrong"
# subject = "_heapq.heappush(heap: list)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_heapq.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _heapq.heappush(heap: list); call it with the wrong type.

typeshed contract: heap is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _heapq import heappush
try:
    heappush(12345, None)  # heap: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_heapq/heappushpop__heap_as_list_wrong.py`.
#[test]
fn test_gen_type_std_libs__heapq_heappushpop__heap_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_heapq"
# dimension = "type"
# case = "heappushpop__heap_as_list_wrong"
# subject = "_heapq.heappushpop(heap: list)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_heapq.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _heapq.heappushpop(heap: list); call it with the wrong type.

typeshed contract: heap is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _heapq import heappushpop
try:
    heappushpop(12345, None)  # heap: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_heapq/heapreplace__heap_as_list_wrong.py`.
#[test]
fn test_gen_type_std_libs__heapq_heapreplace__heap_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_heapq"
# dimension = "type"
# case = "heapreplace__heap_as_list_wrong"
# subject = "_heapq.heapreplace(heap: list)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_heapq.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _heapq.heapreplace(heap: list); call it with the wrong type.

typeshed contract: heap is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _heapq import heapreplace
try:
    heapreplace(12345, None)  # heap: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/heapq/nlargest__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_heapq_nlargest__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "type"
# case = "nlargest__n_as_int_wrong"
# subject = "heapq.nlargest(n: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/heapq.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: heapq.nlargest(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from heapq import nlargest
try:
    nlargest("not_an_int", None)  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/heapq/nsmallest__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_heapq_nsmallest__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "type"
# case = "nsmallest__n_as_int_wrong"
# subject = "heapq.nsmallest(n: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/heapq.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: heapq.nsmallest(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from heapq import nsmallest
try:
    nsmallest("not_an_int", None)  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
