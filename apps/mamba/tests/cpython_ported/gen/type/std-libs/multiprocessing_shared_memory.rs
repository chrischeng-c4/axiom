use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/multiprocessing_shared_memory/ShareableList____getitem____position_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_shared_memory_ShareableList____getitem____position_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_shared_memory"
# dimension = "type"
# case = "ShareableList____getitem____position_as_int_wrong"
# subject = "multiprocessing.shared_memory.ShareableList.__getitem__(position: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/shared_memory.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.shared_memory.ShareableList.__getitem__(position: int); call it with the wrong type.

typeshed contract: position is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.shared_memory import ShareableList
obj = object.__new__(ShareableList)
try:
    obj.__getitem__("not_an_int")  # position: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_shared_memory/ShareableList____setitem____position_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_shared_memory_ShareableList____setitem____position_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_shared_memory"
# dimension = "type"
# case = "ShareableList____setitem____position_as_int_wrong"
# subject = "multiprocessing.shared_memory.ShareableList.__setitem__(position: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/shared_memory.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.shared_memory.ShareableList.__setitem__(position: int); call it with the wrong type.

typeshed contract: position is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.shared_memory import ShareableList
obj = object.__new__(ShareableList)
try:
    obj.__setitem__("not_an_int", None)  # position: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_shared_memory/ShareableList__init__sequence_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_shared_memory_ShareableList__init__sequence_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_shared_memory"
# dimension = "type"
# case = "ShareableList__init__sequence_as_Iterable_wrong"
# subject = "multiprocessing.shared_memory.ShareableList.__init__(sequence: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/shared_memory.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.shared_memory.ShareableList.__init__(sequence: Iterable); call it with the wrong type.

typeshed contract: sequence is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.shared_memory import ShareableList
try:
    ShareableList(_W())  # sequence: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_shared_memory/ShareableList__init__sequence_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_shared_memory_ShareableList__init__sequence_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_shared_memory"
# dimension = "type"
# case = "ShareableList__init__sequence_as_typed_wrong"
# subject = "multiprocessing.shared_memory.ShareableList.__init__(sequence: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/shared_memory.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.shared_memory.ShareableList.__init__(sequence: typed); call it with the wrong type.

typeshed contract: sequence is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.shared_memory import ShareableList
try:
    ShareableList(_W())  # sequence: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_shared_memory/SharedMemory__init__name_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_shared_memory_SharedMemory__init__name_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_shared_memory"
# dimension = "type"
# case = "SharedMemory__init__name_as_typed_wrong"
# subject = "multiprocessing.shared_memory.SharedMemory.__init__(name: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/shared_memory.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.shared_memory.SharedMemory.__init__(name: typed); call it with the wrong type.

typeshed contract: name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.shared_memory import SharedMemory
try:
    SharedMemory(_W())  # name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
