use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_interpqueues/bind__qid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpqueues_bind__qid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpqueues"
# dimension = "type"
# case = "bind__qid_as_SupportsIndex_wrong"
# subject = "_interpqueues.bind(qid: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpqueues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpqueues.bind(qid: SupportsIndex); call it with the wrong type.

typeshed contract: qid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpqueues import bind
try:
    bind(_W())  # qid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpqueues/create__maxsize_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpqueues_create__maxsize_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpqueues"
# dimension = "type"
# case = "create__maxsize_as_SupportsIndex_wrong"
# subject = "_interpqueues.create(maxsize: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpqueues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpqueues.create(maxsize: SupportsIndex); call it with the wrong type.

typeshed contract: maxsize is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpqueues import create
try:
    create(_W())  # maxsize: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpqueues/destroy__qid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpqueues_destroy__qid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpqueues"
# dimension = "type"
# case = "destroy__qid_as_SupportsIndex_wrong"
# subject = "_interpqueues.destroy(qid: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpqueues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpqueues.destroy(qid: SupportsIndex); call it with the wrong type.

typeshed contract: qid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpqueues import destroy
try:
    destroy(_W())  # qid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpqueues/get__qid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpqueues_get__qid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpqueues"
# dimension = "type"
# case = "get__qid_as_SupportsIndex_wrong"
# subject = "_interpqueues.get(qid: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpqueues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpqueues.get(qid: SupportsIndex); call it with the wrong type.

typeshed contract: qid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpqueues import get
try:
    get(_W())  # qid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpqueues/get_count__qid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpqueues_get_count__qid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpqueues"
# dimension = "type"
# case = "get_count__qid_as_SupportsIndex_wrong"
# subject = "_interpqueues.get_count(qid: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpqueues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpqueues.get_count(qid: SupportsIndex); call it with the wrong type.

typeshed contract: qid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpqueues import get_count
try:
    get_count(_W())  # qid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpqueues/get_maxsize__qid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpqueues_get_maxsize__qid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpqueues"
# dimension = "type"
# case = "get_maxsize__qid_as_SupportsIndex_wrong"
# subject = "_interpqueues.get_maxsize(qid: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpqueues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpqueues.get_maxsize(qid: SupportsIndex); call it with the wrong type.

typeshed contract: qid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpqueues import get_maxsize
try:
    get_maxsize(_W())  # qid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpqueues/get_queue_defaults__qid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpqueues_get_queue_defaults__qid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpqueues"
# dimension = "type"
# case = "get_queue_defaults__qid_as_SupportsIndex_wrong"
# subject = "_interpqueues.get_queue_defaults(qid: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpqueues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpqueues.get_queue_defaults(qid: SupportsIndex); call it with the wrong type.

typeshed contract: qid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpqueues import get_queue_defaults
try:
    get_queue_defaults(_W())  # qid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpqueues/is_full__qid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpqueues_is_full__qid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpqueues"
# dimension = "type"
# case = "is_full__qid_as_SupportsIndex_wrong"
# subject = "_interpqueues.is_full(qid: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpqueues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpqueues.is_full(qid: SupportsIndex); call it with the wrong type.

typeshed contract: qid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpqueues import is_full
try:
    is_full(_W())  # qid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpqueues/put__qid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpqueues_put__qid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpqueues"
# dimension = "type"
# case = "put__qid_as_SupportsIndex_wrong"
# subject = "_interpqueues.put(qid: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpqueues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpqueues.put(qid: SupportsIndex); call it with the wrong type.

typeshed contract: qid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpqueues import put
try:
    put(_W(), None)  # qid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpqueues/release__qid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpqueues_release__qid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpqueues"
# dimension = "type"
# case = "release__qid_as_SupportsIndex_wrong"
# subject = "_interpqueues.release(qid: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpqueues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpqueues.release(qid: SupportsIndex); call it with the wrong type.

typeshed contract: qid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpqueues import release
try:
    release(_W())  # qid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
