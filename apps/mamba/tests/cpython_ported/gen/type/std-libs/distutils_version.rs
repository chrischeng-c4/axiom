use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/distutils_version/LooseVersion__init__vstring_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_version_LooseVersion__init__vstring_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_version"
# dimension = "type"
# case = "LooseVersion__init__vstring_as_typed_wrong"
# subject = "distutils.version.LooseVersion.__init__(vstring: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/version.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.version.LooseVersion.__init__(vstring: typed); call it with the wrong type.

typeshed contract: vstring is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.version import LooseVersion
try:
    LooseVersion(_W())  # vstring: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_version/LooseVersion__parse__vstring_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_version_LooseVersion__parse__vstring_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_version"
# dimension = "type"
# case = "LooseVersion__parse__vstring_as_str_wrong"
# subject = "distutils.version.LooseVersion.parse(vstring: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/version.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.version.LooseVersion.parse(vstring: str); call it with the wrong type.

typeshed contract: vstring is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.version import LooseVersion
obj = object.__new__(LooseVersion)
try:
    obj.parse(12345)  # vstring: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_version/StrictVersion__init__vstring_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_version_StrictVersion__init__vstring_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_version"
# dimension = "type"
# case = "StrictVersion__init__vstring_as_typed_wrong"
# subject = "distutils.version.StrictVersion.__init__(vstring: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/version.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.version.StrictVersion.__init__(vstring: typed); call it with the wrong type.

typeshed contract: vstring is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.version import StrictVersion
try:
    StrictVersion(_W())  # vstring: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_version/StrictVersion__parse__vstring_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_version_StrictVersion__parse__vstring_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_version"
# dimension = "type"
# case = "StrictVersion__parse__vstring_as_str_wrong"
# subject = "distutils.version.StrictVersion.parse(vstring: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/version.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.version.StrictVersion.parse(vstring: str); call it with the wrong type.

typeshed contract: vstring is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.version import StrictVersion
obj = object.__new__(StrictVersion)
try:
    obj.parse(12345)  # vstring: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_version/Version__init__vstring_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_version_Version__init__vstring_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_version"
# dimension = "type"
# case = "Version__init__vstring_as_typed_wrong"
# subject = "distutils.version.Version.__init__(vstring: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/version.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.version.Version.__init__(vstring: typed); call it with the wrong type.

typeshed contract: vstring is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.version import Version
try:
    Version(_W())  # vstring: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_version/Version__parse__vstring_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_version_Version__parse__vstring_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_version"
# dimension = "type"
# case = "Version__parse__vstring_as_str_wrong"
# subject = "distutils.version.Version.parse(vstring: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/version.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.version.Version.parse(vstring: str); call it with the wrong type.

typeshed contract: vstring is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.version import Version
obj = object.__new__(Version)
try:
    obj.parse(12345)  # vstring: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
