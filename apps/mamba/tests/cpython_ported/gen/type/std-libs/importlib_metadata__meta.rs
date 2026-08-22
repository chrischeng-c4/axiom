use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/importlib_metadata__meta/PackageMetadata____contains____item_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_metadata__meta_PackageMetadata____contains____item_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_metadata__meta"
# dimension = "type"
# case = "PackageMetadata____contains____item_as_str_wrong"
# subject = "importlib.metadata._meta.PackageMetadata.__contains__(item: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/metadata/_meta.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.metadata._meta.PackageMetadata.__contains__(item: str); call it with the wrong type.

typeshed contract: item is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.metadata._meta import PackageMetadata
obj = object.__new__(PackageMetadata)
try:
    obj.__contains__(12345)  # item: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_metadata__meta/PackageMetadata____getitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_metadata__meta_PackageMetadata____getitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_metadata__meta"
# dimension = "type"
# case = "PackageMetadata____getitem____key_as_str_wrong"
# subject = "importlib.metadata._meta.PackageMetadata.__getitem__(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/metadata/_meta.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.metadata._meta.PackageMetadata.__getitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.metadata._meta import PackageMetadata
obj = object.__new__(PackageMetadata)
try:
    obj.__getitem__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_metadata__meta/PackageMetadata__get__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_metadata__meta_PackageMetadata__get__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_metadata__meta"
# dimension = "type"
# case = "PackageMetadata__get__name_as_str_wrong"
# subject = "importlib.metadata._meta.PackageMetadata.get(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/metadata/_meta.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.metadata._meta.PackageMetadata.get(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.metadata._meta import PackageMetadata
obj = object.__new__(PackageMetadata)
try:
    obj.get(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_metadata__meta/PackageMetadata__get_all__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_metadata__meta_PackageMetadata__get_all__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_metadata__meta"
# dimension = "type"
# case = "PackageMetadata__get_all__name_as_str_wrong"
# subject = "importlib.metadata._meta.PackageMetadata.get_all(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/metadata/_meta.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.metadata._meta.PackageMetadata.get_all(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.metadata._meta import PackageMetadata
obj = object.__new__(PackageMetadata)
try:
    obj.get_all(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_metadata__meta/SimplePath____truediv____other_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_metadata__meta_SimplePath____truediv____other_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_metadata__meta"
# dimension = "type"
# case = "SimplePath____truediv____other_as_StrPath_wrong"
# subject = "importlib.metadata._meta.SimplePath.__truediv__(other: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/metadata/_meta.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.metadata._meta.SimplePath.__truediv__(other: StrPath); call it with the wrong type.

typeshed contract: other is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.metadata._meta import SimplePath
obj = object.__new__(SimplePath)
try:
    obj.__truediv__(_W())  # other: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_metadata__meta/SimplePath____truediv____other_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_metadata__meta_SimplePath____truediv____other_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_metadata__meta"
# dimension = "type"
# case = "SimplePath____truediv____other_as_str_wrong"
# subject = "importlib.metadata._meta.SimplePath.__truediv__(other: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/metadata/_meta.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.metadata._meta.SimplePath.__truediv__(other: str); call it with the wrong type.

typeshed contract: other is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.metadata._meta import SimplePath
obj = object.__new__(SimplePath)
try:
    obj.__truediv__(12345)  # other: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_metadata__meta/SimplePath__joinpath__other_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_metadata__meta_SimplePath__joinpath__other_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_metadata__meta"
# dimension = "type"
# case = "SimplePath__joinpath__other_as_StrPath_wrong"
# subject = "importlib.metadata._meta.SimplePath.joinpath(other: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/metadata/_meta.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.metadata._meta.SimplePath.joinpath(other: StrPath); call it with the wrong type.

typeshed contract: other is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.metadata._meta import SimplePath
obj = object.__new__(SimplePath)
try:
    obj.joinpath(_W())  # other: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_metadata__meta/SimplePath__joinpath__other_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_metadata__meta_SimplePath__joinpath__other_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_metadata__meta"
# dimension = "type"
# case = "SimplePath__joinpath__other_as_str_wrong"
# subject = "importlib.metadata._meta.SimplePath.joinpath(other: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/metadata/_meta.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.metadata._meta.SimplePath.joinpath(other: str); call it with the wrong type.

typeshed contract: other is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.metadata._meta import SimplePath
obj = object.__new__(SimplePath)
try:
    obj.joinpath(12345)  # other: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
