use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/glob/escape__pathname_as_AnyStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_glob_escape__pathname_as_AnyStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "type"
# case = "escape__pathname_as_AnyStr_wrong"
# subject = "glob.escape(pathname: AnyStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: glob.escape(pathname: AnyStr); call it with the wrong type.

typeshed contract: pathname is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from glob import escape
try:
    escape(_W())  # pathname: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/glob/glob0__dirname_as_AnyStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_glob_glob0__dirname_as_AnyStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "type"
# case = "glob0__dirname_as_AnyStr_wrong"
# subject = "glob.glob0(dirname: AnyStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: glob.glob0(dirname: AnyStr); call it with the wrong type.

typeshed contract: dirname is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from glob import glob0
try:
    glob0(_W(), None)  # dirname: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/glob/glob1__dirname_as_AnyStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_glob_glob1__dirname_as_AnyStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "type"
# case = "glob1__dirname_as_AnyStr_wrong"
# subject = "glob.glob1(dirname: AnyStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: glob.glob1(dirname: AnyStr); call it with the wrong type.

typeshed contract: dirname is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from glob import glob1
try:
    glob1(_W(), None)  # dirname: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/glob/glob__pathname_as_AnyStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_glob_glob__pathname_as_AnyStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "type"
# case = "glob__pathname_as_AnyStr_wrong"
# subject = "glob.glob(pathname: AnyStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: glob.glob(pathname: AnyStr); call it with the wrong type.

typeshed contract: pathname is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from glob import glob
try:
    glob(_W())  # pathname: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/glob/has_magic__s_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_glob_has_magic__s_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "type"
# case = "has_magic__s_as_typed_wrong"
# subject = "glob.has_magic(s: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: glob.has_magic(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from glob import has_magic
try:
    has_magic(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/glob/iglob__pathname_as_AnyStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_glob_iglob__pathname_as_AnyStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "type"
# case = "iglob__pathname_as_AnyStr_wrong"
# subject = "glob.iglob(pathname: AnyStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: glob.iglob(pathname: AnyStr); call it with the wrong type.

typeshed contract: pathname is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from glob import iglob
try:
    iglob(_W())  # pathname: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/glob/translate__pat_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_glob_translate__pat_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "type"
# case = "translate__pat_as_str_wrong"
# subject = "glob.translate(pat: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: glob.translate(pat: str); call it with the wrong type.

typeshed contract: pat is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from glob import translate
try:
    translate(12345)  # pat: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
