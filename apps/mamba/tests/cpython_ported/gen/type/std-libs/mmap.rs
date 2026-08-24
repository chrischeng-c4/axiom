use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/mmap/mmap____buffer____flags_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap____buffer____flags_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap____buffer____flags_as_int_wrong"
# subject = "mmap.mmap.__buffer__(flags: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.__buffer__(flags: int); call it with the wrong type.

typeshed contract: flags is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.__buffer__("not_an_int")  # flags: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap____delitem____key_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap____delitem____key_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap____delitem____key_as_typed_wrong"
# subject = "mmap.mmap.__delitem__(key: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.__delitem__(key: typed); call it with the wrong type.

typeshed contract: key is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.__delitem__(_W())  # key: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap____getitem____key_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap____getitem____key_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap____getitem____key_as_SupportsIndex_wrong"
# subject = "mmap.mmap.__getitem__(key: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.__getitem__(key: SupportsIndex); call it with the wrong type.

typeshed contract: key is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.__getitem__(_W())  # key: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap____getitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap____getitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap____getitem____key_as_slice_wrong"
# subject = "mmap.mmap.__getitem__(key: slice)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.__getitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.__getitem__(_W())  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap____new____fileno_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap____new____fileno_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap____new____fileno_as_int_wrong"
# subject = "mmap.mmap.__new__(fileno: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.__new__(fileno: int); call it with the wrong type.

typeshed contract: fileno is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.__new__("not_an_int", 0)  # fileno: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap____release_buffer____buffer_as_memoryview_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap____release_buffer____buffer_as_memoryview_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap____release_buffer____buffer_as_memoryview_wrong"
# subject = "mmap.mmap.__release_buffer__(buffer: memoryview)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.__release_buffer__(buffer: memoryview); call it with the wrong type.

typeshed contract: buffer is memoryview. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.__release_buffer__(12345)  # buffer: memoryview <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap____setitem____key_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap____setitem____key_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap____setitem____key_as_SupportsIndex_wrong"
# subject = "mmap.mmap.__setitem__(key: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.__setitem__(key: SupportsIndex); call it with the wrong type.

typeshed contract: key is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.__setitem__(_W(), 0)  # key: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap____setitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap____setitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap____setitem____key_as_slice_wrong"
# subject = "mmap.mmap.__setitem__(key: slice)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.__setitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.__setitem__(_W(), None)  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap__find__view_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap__find__view_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap__find__view_as_ReadableBuffer_wrong"
# subject = "mmap.mmap.find(view: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.find(view: ReadableBuffer); call it with the wrong type.

typeshed contract: view is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.find(_W())  # view: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap__flush__offset_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap__flush__offset_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap__flush__offset_as_int_wrong"
# subject = "mmap.mmap.flush(offset: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.flush(offset: int); call it with the wrong type.

typeshed contract: offset is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.flush("not_an_int")  # offset: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap__madvise__option_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap__madvise__option_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap__madvise__option_as_int_wrong"
# subject = "mmap.mmap.madvise(option: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.madvise(option: int); call it with the wrong type.

typeshed contract: option is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.madvise("not_an_int")  # option: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap__move__dest_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap__move__dest_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap__move__dest_as_int_wrong"
# subject = "mmap.mmap.move(dest: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.move(dest: int); call it with the wrong type.

typeshed contract: dest is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.move("not_an_int", 0, 0)  # dest: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap__read__n_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap__read__n_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap__read__n_as_typed_wrong"
# subject = "mmap.mmap.read(n: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.read(n: typed); call it with the wrong type.

typeshed contract: n is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.read(_W())  # n: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap__resize__newsize_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap__resize__newsize_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap__resize__newsize_as_int_wrong"
# subject = "mmap.mmap.resize(newsize: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.resize(newsize: int); call it with the wrong type.

typeshed contract: newsize is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.resize("not_an_int")  # newsize: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap__rfind__view_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap__rfind__view_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap__rfind__view_as_ReadableBuffer_wrong"
# subject = "mmap.mmap.rfind(view: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.rfind(view: ReadableBuffer); call it with the wrong type.

typeshed contract: view is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.rfind(_W())  # view: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap__seek__pos_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap__seek__pos_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap__seek__pos_as_int_wrong"
# subject = "mmap.mmap.seek(pos: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.seek(pos: int); call it with the wrong type.

typeshed contract: pos is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.seek("not_an_int")  # pos: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap__set_name__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap__set_name__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap__set_name__name_as_str_wrong"
# subject = "mmap.mmap.set_name(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.set_name(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.set_name(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap__write__bytes_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap__write__bytes_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap__write__bytes_as_ReadableBuffer_wrong"
# subject = "mmap.mmap.write(bytes: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.write(bytes: ReadableBuffer); call it with the wrong type.

typeshed contract: bytes is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.write(_W())  # bytes: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mmap/mmap__write_byte__byte_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_mmap_mmap__write_byte__byte_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap__write_byte__byte_as_int_wrong"
# subject = "mmap.mmap.write_byte(byte: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.write_byte(byte: int); call it with the wrong type.

typeshed contract: byte is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.write_byte("not_an_int")  # byte: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
