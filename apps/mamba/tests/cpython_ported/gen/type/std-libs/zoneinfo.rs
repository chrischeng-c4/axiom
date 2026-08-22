use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/zoneinfo/ZoneInfo____new____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zoneinfo_ZoneInfo____new____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "type"
# case = "ZoneInfo____new____key_as_str_wrong"
# subject = "zoneinfo.ZoneInfo.__new__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zoneinfo.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zoneinfo.ZoneInfo.__new__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zoneinfo import ZoneInfo
obj = object.__new__(ZoneInfo)
try:
    obj.__new__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zoneinfo/ZoneInfo__dst__dt_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zoneinfo_ZoneInfo__dst__dt_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "type"
# case = "ZoneInfo__dst__dt_as_typed_wrong"
# subject = "zoneinfo.ZoneInfo.dst(dt: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zoneinfo.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zoneinfo.ZoneInfo.dst(dt: typed); call it with the wrong type.

typeshed contract: dt is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zoneinfo import ZoneInfo
obj = object.__new__(ZoneInfo)
try:
    obj.dst(_W())  # dt: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zoneinfo/ZoneInfo__from_file__file_obj_as__IOBytes_wrong.py`.
#[test]
fn test_gen_type_std_libs_zoneinfo_ZoneInfo__from_file__file_obj_as__IOBytes_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "type"
# case = "ZoneInfo__from_file__file_obj_as__IOBytes_wrong"
# subject = "zoneinfo.ZoneInfo.from_file(file_obj: _IOBytes)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zoneinfo.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zoneinfo.ZoneInfo.from_file(file_obj: _IOBytes); call it with the wrong type.

typeshed contract: file_obj is _IOBytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zoneinfo import ZoneInfo
try:
    ZoneInfo.from_file(_W())  # file_obj: _IOBytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zoneinfo/ZoneInfo__no_cache__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zoneinfo_ZoneInfo__no_cache__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "type"
# case = "ZoneInfo__no_cache__key_as_str_wrong"
# subject = "zoneinfo.ZoneInfo.no_cache(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zoneinfo.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zoneinfo.ZoneInfo.no_cache(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zoneinfo import ZoneInfo
try:
    ZoneInfo.no_cache(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zoneinfo/ZoneInfo__tzname__dt_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zoneinfo_ZoneInfo__tzname__dt_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "type"
# case = "ZoneInfo__tzname__dt_as_typed_wrong"
# subject = "zoneinfo.ZoneInfo.tzname(dt: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zoneinfo.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zoneinfo.ZoneInfo.tzname(dt: typed); call it with the wrong type.

typeshed contract: dt is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zoneinfo import ZoneInfo
obj = object.__new__(ZoneInfo)
try:
    obj.tzname(_W())  # dt: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zoneinfo/ZoneInfo__utcoffset__dt_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zoneinfo_ZoneInfo__utcoffset__dt_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "type"
# case = "ZoneInfo__utcoffset__dt_as_typed_wrong"
# subject = "zoneinfo.ZoneInfo.utcoffset(dt: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zoneinfo.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zoneinfo.ZoneInfo.utcoffset(dt: typed); call it with the wrong type.

typeshed contract: dt is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zoneinfo import ZoneInfo
obj = object.__new__(ZoneInfo)
try:
    obj.utcoffset(_W())  # dt: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
