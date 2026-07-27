use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/Array__typecode_or_type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_Array__typecode_or_type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "Array__typecode_or_type_as_str_wrong"
# subject = "multiprocessing.sharedctypes.Array(typecode_or_type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.Array(typecode_or_type: str); call it with the wrong type.

typeshed contract: typecode_or_type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.sharedctypes import Array
try:
    Array(12345, None)  # typecode_or_type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/Array__typecode_or_type_as_type_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_Array__typecode_or_type_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "Array__typecode_or_type_as_type_wrong"
# subject = "multiprocessing.sharedctypes.Array(typecode_or_type: type)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.Array(typecode_or_type: type); call it with the wrong type.

typeshed contract: typecode_or_type is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import Array
try:
    Array(_W(), None)  # typecode_or_type: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/Array__typecode_or_type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_Array__typecode_or_type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "Array__typecode_or_type_as_typed_wrong"
# subject = "multiprocessing.sharedctypes.Array(typecode_or_type: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.Array(typecode_or_type: typed); call it with the wrong type.

typeshed contract: typecode_or_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import Array
try:
    Array(_W(), None)  # typecode_or_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/RawArray__typecode_or_type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_RawArray__typecode_or_type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "RawArray__typecode_or_type_as_str_wrong"
# subject = "multiprocessing.sharedctypes.RawArray(typecode_or_type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.RawArray(typecode_or_type: str); call it with the wrong type.

typeshed contract: typecode_or_type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.sharedctypes import RawArray
try:
    RawArray(12345, None)  # typecode_or_type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/RawArray__typecode_or_type_as_type_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_RawArray__typecode_or_type_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "RawArray__typecode_or_type_as_type_wrong"
# subject = "multiprocessing.sharedctypes.RawArray(typecode_or_type: type)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.RawArray(typecode_or_type: type); call it with the wrong type.

typeshed contract: typecode_or_type is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import RawArray
try:
    RawArray(_W(), None)  # typecode_or_type: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/RawValue__typecode_or_type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_RawValue__typecode_or_type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "RawValue__typecode_or_type_as_str_wrong"
# subject = "multiprocessing.sharedctypes.RawValue(typecode_or_type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.RawValue(typecode_or_type: str); call it with the wrong type.

typeshed contract: typecode_or_type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.sharedctypes import RawValue
try:
    RawValue(12345)  # typecode_or_type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/RawValue__typecode_or_type_as_type_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_RawValue__typecode_or_type_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "RawValue__typecode_or_type_as_type_wrong"
# subject = "multiprocessing.sharedctypes.RawValue(typecode_or_type: type)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.RawValue(typecode_or_type: type); call it with the wrong type.

typeshed contract: typecode_or_type is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import RawValue
try:
    RawValue(_W())  # typecode_or_type: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedArray____getitem____i_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedArray____getitem____i_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedArray____getitem____i_as_SupportsIndex_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedArray.__getitem__(i: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedArray.__getitem__(i: SupportsIndex); call it with the wrong type.

typeshed contract: i is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedArray
obj = object.__new__(SynchronizedArray)
try:
    obj.__getitem__(_W())  # i: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedArray____getitem____i_as_slice_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedArray____getitem____i_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedArray____getitem____i_as_slice_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedArray.__getitem__(i: slice)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedArray.__getitem__(i: slice); call it with the wrong type.

typeshed contract: i is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedArray
obj = object.__new__(SynchronizedArray)
try:
    obj.__getitem__(_W())  # i: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedArray____getslice____start_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedArray____getslice____start_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedArray____getslice____start_as_SupportsIndex_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedArray.__getslice__(start: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedArray.__getslice__(start: SupportsIndex); call it with the wrong type.

typeshed contract: start is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedArray
obj = object.__new__(SynchronizedArray)
try:
    obj.__getslice__(_W(), None)  # start: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedArray____setitem____i_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedArray____setitem____i_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedArray____setitem____i_as_SupportsIndex_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedArray.__setitem__(i: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedArray.__setitem__(i: SupportsIndex); call it with the wrong type.

typeshed contract: i is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedArray
obj = object.__new__(SynchronizedArray)
try:
    obj.__setitem__(_W(), None)  # i: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedArray____setitem____i_as_slice_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedArray____setitem____i_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedArray____setitem____i_as_slice_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedArray.__setitem__(i: slice)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedArray.__setitem__(i: slice); call it with the wrong type.

typeshed contract: i is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedArray
obj = object.__new__(SynchronizedArray)
try:
    obj.__setitem__(_W(), None)  # i: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedArray____setslice____start_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedArray____setslice____start_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedArray____setslice____start_as_SupportsIndex_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedArray.__setslice__(start: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedArray.__setslice__(start: SupportsIndex); call it with the wrong type.

typeshed contract: start is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedArray
obj = object.__new__(SynchronizedArray)
try:
    obj.__setslice__(_W(), None, None)  # start: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedBase____exit____exc_type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedBase____exit____exc_type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedBase____exit____exc_type_as_typed_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedBase.__exit__(exc_type: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedBase.__exit__(exc_type: typed); call it with the wrong type.

typeshed contract: exc_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedBase
obj = object.__new__(SynchronizedBase)
try:
    obj.__exit__(_W(), None, None)  # exc_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedBase__init__lock_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedBase__init__lock_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedBase__init__lock_as_typed_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedBase.__init__(lock: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedBase.__init__(lock: typed); call it with the wrong type.

typeshed contract: lock is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedBase
try:
    SynchronizedBase(None, _W())  # lock: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedString____getitem____i_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedString____getitem____i_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedString____getitem____i_as_SupportsIndex_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedString.__getitem__(i: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedString.__getitem__(i: SupportsIndex); call it with the wrong type.

typeshed contract: i is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedString
obj = object.__new__(SynchronizedString)
try:
    obj.__getitem__(_W())  # i: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedString____getitem____i_as_slice_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedString____getitem____i_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedString____getitem____i_as_slice_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedString.__getitem__(i: slice)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedString.__getitem__(i: slice); call it with the wrong type.

typeshed contract: i is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedString
obj = object.__new__(SynchronizedString)
try:
    obj.__getitem__(_W())  # i: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedString____getslice____start_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedString____getslice____start_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedString____getslice____start_as_SupportsIndex_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedString.__getslice__(start: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedString.__getslice__(start: SupportsIndex); call it with the wrong type.

typeshed contract: start is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedString
obj = object.__new__(SynchronizedString)
try:
    obj.__getslice__(_W(), None)  # start: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedString____setitem____i_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedString____setitem____i_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedString____setitem____i_as_SupportsIndex_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedString.__setitem__(i: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedString.__setitem__(i: SupportsIndex); call it with the wrong type.

typeshed contract: i is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedString
obj = object.__new__(SynchronizedString)
try:
    obj.__setitem__(_W(), b"")  # i: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedString____setitem____i_as_slice_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedString____setitem____i_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedString____setitem____i_as_slice_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedString.__setitem__(i: slice)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedString.__setitem__(i: slice); call it with the wrong type.

typeshed contract: i is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedString
obj = object.__new__(SynchronizedString)
try:
    obj.__setitem__(_W(), b"")  # i: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/SynchronizedString____setslice____start_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_SynchronizedString____setslice____start_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedString____setslice____start_as_SupportsIndex_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedString.__setslice__(start: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedString.__setslice__(start: SupportsIndex); call it with the wrong type.

typeshed contract: start is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedString
obj = object.__new__(SynchronizedString)
try:
    obj.__setslice__(_W(), None, b"")  # start: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/Value__typecode_or_type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_Value__typecode_or_type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "Value__typecode_or_type_as_str_wrong"
# subject = "multiprocessing.sharedctypes.Value(typecode_or_type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.Value(typecode_or_type: str); call it with the wrong type.

typeshed contract: typecode_or_type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.sharedctypes import Value
try:
    Value(12345)  # typecode_or_type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/Value__typecode_or_type_as_type_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_Value__typecode_or_type_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "Value__typecode_or_type_as_type_wrong"
# subject = "multiprocessing.sharedctypes.Value(typecode_or_type: type)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.Value(typecode_or_type: type); call it with the wrong type.

typeshed contract: typecode_or_type is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import Value
try:
    Value(_W())  # typecode_or_type: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/Value__typecode_or_type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_Value__typecode_or_type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "Value__typecode_or_type_as_typed_wrong"
# subject = "multiprocessing.sharedctypes.Value(typecode_or_type: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.Value(typecode_or_type: typed); call it with the wrong type.

typeshed contract: typecode_or_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import Value
try:
    Value(_W())  # typecode_or_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/copy__obj_as__CT_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_copy__obj_as__CT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "copy__obj_as__CT_wrong"
# subject = "multiprocessing.sharedctypes.copy(obj: _CT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.copy(obj: _CT); call it with the wrong type.

typeshed contract: obj is _CT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import copy
try:
    copy(_W())  # obj: _CT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/synchronized__obj_as__CT_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_synchronized__obj_as__CT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "synchronized__obj_as__CT_wrong"
# subject = "multiprocessing.sharedctypes.synchronized(obj: _CT)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.synchronized(obj: _CT); call it with the wrong type.

typeshed contract: obj is _CT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import synchronized
try:
    synchronized(_W())  # obj: _CT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/synchronized__obj_as__SimpleCData_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_synchronized__obj_as__SimpleCData_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "synchronized__obj_as__SimpleCData_wrong"
# subject = "multiprocessing.sharedctypes.synchronized(obj: _SimpleCData)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.synchronized(obj: _SimpleCData); call it with the wrong type.

typeshed contract: obj is _SimpleCData. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import synchronized
try:
    synchronized(_W())  # obj: _SimpleCData <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_sharedctypes/synchronized__obj_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_sharedctypes_synchronized__obj_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "synchronized__obj_as_typed_wrong"
# subject = "multiprocessing.sharedctypes.synchronized(obj: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.synchronized(obj: typed); call it with the wrong type.

typeshed contract: obj is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import synchronized
try:
    synchronized(_W())  # obj: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
