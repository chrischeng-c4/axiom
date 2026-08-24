use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/zipfile__path_glob/Translator__extend__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_glob_Translator__extend__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path_glob"
# dimension = "type"
# case = "Translator__extend__pattern_as_str_wrong"
# subject = "zipfile._path.glob.Translator.extend(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.glob.Translator.extend(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path.glob import Translator
obj = object.__new__(Translator)
try:
    obj.extend(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path_glob/Translator__match_dirs__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_glob_Translator__match_dirs__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path_glob"
# dimension = "type"
# case = "Translator__match_dirs__pattern_as_str_wrong"
# subject = "zipfile._path.glob.Translator.match_dirs(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.glob.Translator.match_dirs(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path.glob import Translator
obj = object.__new__(Translator)
try:
    obj.match_dirs(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path_glob/Translator__restrict_rglob__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_glob_Translator__restrict_rglob__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path_glob"
# dimension = "type"
# case = "Translator__restrict_rglob__pattern_as_str_wrong"
# subject = "zipfile._path.glob.Translator.restrict_rglob(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.glob.Translator.restrict_rglob(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path.glob import Translator
obj = object.__new__(Translator)
try:
    obj.restrict_rglob(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path_glob/Translator__star_not_empty__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_glob_Translator__star_not_empty__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path_glob"
# dimension = "type"
# case = "Translator__star_not_empty__pattern_as_str_wrong"
# subject = "zipfile._path.glob.Translator.star_not_empty(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.glob.Translator.star_not_empty(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path.glob import Translator
obj = object.__new__(Translator)
try:
    obj.star_not_empty(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path_glob/Translator__translate__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_glob_Translator__translate__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path_glob"
# dimension = "type"
# case = "Translator__translate__pattern_as_str_wrong"
# subject = "zipfile._path.glob.Translator.translate(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.glob.Translator.translate(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path.glob import Translator
obj = object.__new__(Translator)
try:
    obj.translate(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path_glob/Translator__translate_core__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_glob_Translator__translate_core__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path_glob"
# dimension = "type"
# case = "Translator__translate_core__pattern_as_str_wrong"
# subject = "zipfile._path.glob.Translator.translate_core(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.glob.Translator.translate_core(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path.glob import Translator
obj = object.__new__(Translator)
try:
    obj.translate_core(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path_glob/match_dirs__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_glob_match_dirs__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path_glob"
# dimension = "type"
# case = "match_dirs__pattern_as_str_wrong"
# subject = "zipfile._path.glob.match_dirs(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.glob.match_dirs(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path.glob import match_dirs
try:
    match_dirs(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path_glob/separate__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_glob_separate__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path_glob"
# dimension = "type"
# case = "separate__pattern_as_str_wrong"
# subject = "zipfile._path.glob.separate(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.glob.separate(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path.glob import separate
try:
    separate(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path_glob/translate__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_glob_translate__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path_glob"
# dimension = "type"
# case = "translate__pattern_as_str_wrong"
# subject = "zipfile._path.glob.translate(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.glob.translate(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path.glob import translate
try:
    translate(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path_glob/translate_core__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_glob_translate_core__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path_glob"
# dimension = "type"
# case = "translate_core__pattern_as_str_wrong"
# subject = "zipfile._path.glob.translate_core(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.glob.translate_core(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path.glob import translate_core
try:
    translate_core(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
