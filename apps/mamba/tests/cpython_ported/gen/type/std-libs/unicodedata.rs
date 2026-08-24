use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/unicodedata/UCD__bidirectional__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_UCD__bidirectional__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "UCD__bidirectional__chr_as_str_wrong"
# subject = "unicodedata.UCD.bidirectional(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.UCD.bidirectional(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import UCD
obj = object.__new__(UCD)
try:
    obj.bidirectional(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/UCD__category__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_UCD__category__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "UCD__category__chr_as_str_wrong"
# subject = "unicodedata.UCD.category(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.UCD.category(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import UCD
obj = object.__new__(UCD)
try:
    obj.category(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/UCD__combining__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_UCD__combining__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "UCD__combining__chr_as_str_wrong"
# subject = "unicodedata.UCD.combining(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.UCD.combining(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import UCD
obj = object.__new__(UCD)
try:
    obj.combining(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/UCD__decimal__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_UCD__decimal__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "UCD__decimal__chr_as_str_wrong"
# subject = "unicodedata.UCD.decimal(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.UCD.decimal(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import UCD
obj = object.__new__(UCD)
try:
    obj.decimal(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/UCD__decomposition__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_UCD__decomposition__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "UCD__decomposition__chr_as_str_wrong"
# subject = "unicodedata.UCD.decomposition(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.UCD.decomposition(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import UCD
obj = object.__new__(UCD)
try:
    obj.decomposition(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/UCD__digit__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_UCD__digit__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "UCD__digit__chr_as_str_wrong"
# subject = "unicodedata.UCD.digit(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.UCD.digit(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import UCD
obj = object.__new__(UCD)
try:
    obj.digit(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/UCD__east_asian_width__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_UCD__east_asian_width__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "UCD__east_asian_width__chr_as_str_wrong"
# subject = "unicodedata.UCD.east_asian_width(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.UCD.east_asian_width(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import UCD
obj = object.__new__(UCD)
try:
    obj.east_asian_width(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/UCD__is_normalized__form_as__NormalizationForm_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_UCD__is_normalized__form_as__NormalizationForm_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "UCD__is_normalized__form_as__NormalizationForm_wrong"
# subject = "unicodedata.UCD.is_normalized(form: _NormalizationForm)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.UCD.is_normalized(form: _NormalizationForm); call it with the wrong type.

typeshed contract: form is _NormalizationForm. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unicodedata import UCD
obj = object.__new__(UCD)
try:
    obj.is_normalized(_W(), "")  # form: _NormalizationForm <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/UCD__lookup__name_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_UCD__lookup__name_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "UCD__lookup__name_as_typed_wrong"
# subject = "unicodedata.UCD.lookup(name: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.UCD.lookup(name: typed); call it with the wrong type.

typeshed contract: name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unicodedata import UCD
obj = object.__new__(UCD)
try:
    obj.lookup(_W())  # name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/UCD__mirrored__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_UCD__mirrored__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "UCD__mirrored__chr_as_str_wrong"
# subject = "unicodedata.UCD.mirrored(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.UCD.mirrored(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import UCD
obj = object.__new__(UCD)
try:
    obj.mirrored(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/UCD__name__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_UCD__name__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "UCD__name__chr_as_str_wrong"
# subject = "unicodedata.UCD.name(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.UCD.name(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import UCD
obj = object.__new__(UCD)
try:
    obj.name(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/UCD__normalize__form_as__NormalizationForm_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_UCD__normalize__form_as__NormalizationForm_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "UCD__normalize__form_as__NormalizationForm_wrong"
# subject = "unicodedata.UCD.normalize(form: _NormalizationForm)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.UCD.normalize(form: _NormalizationForm); call it with the wrong type.

typeshed contract: form is _NormalizationForm. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unicodedata import UCD
obj = object.__new__(UCD)
try:
    obj.normalize(_W(), "")  # form: _NormalizationForm <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/UCD__numeric__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_UCD__numeric__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "UCD__numeric__chr_as_str_wrong"
# subject = "unicodedata.UCD.numeric(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.UCD.numeric(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import UCD
obj = object.__new__(UCD)
try:
    obj.numeric(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/bidirectional__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_bidirectional__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "bidirectional__chr_as_str_wrong"
# subject = "unicodedata.bidirectional(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.bidirectional(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import bidirectional
try:
    bidirectional(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/block__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_block__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "block__chr_as_str_wrong"
# subject = "unicodedata.block(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.block(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import block
try:
    block(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/category__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_category__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "category__chr_as_str_wrong"
# subject = "unicodedata.category(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.category(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import category
try:
    category(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/combining__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_combining__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "combining__chr_as_str_wrong"
# subject = "unicodedata.combining(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.combining(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import combining
try:
    combining(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/decimal__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_decimal__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "decimal__chr_as_str_wrong"
# subject = "unicodedata.decimal(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.decimal(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import decimal
try:
    decimal(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/decomposition__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_decomposition__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "decomposition__chr_as_str_wrong"
# subject = "unicodedata.decomposition(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.decomposition(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import decomposition
try:
    decomposition(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/digit__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_digit__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "digit__chr_as_str_wrong"
# subject = "unicodedata.digit(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.digit(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import digit
try:
    digit(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/east_asian_width__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_east_asian_width__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "east_asian_width__chr_as_str_wrong"
# subject = "unicodedata.east_asian_width(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.east_asian_width(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import east_asian_width
try:
    east_asian_width(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/extended_pictographic__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_extended_pictographic__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "extended_pictographic__chr_as_str_wrong"
# subject = "unicodedata.extended_pictographic(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.extended_pictographic(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import extended_pictographic
try:
    extended_pictographic(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/grapheme_cluster_break__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_grapheme_cluster_break__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "grapheme_cluster_break__chr_as_str_wrong"
# subject = "unicodedata.grapheme_cluster_break(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.grapheme_cluster_break(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import grapheme_cluster_break
try:
    grapheme_cluster_break(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/indic_conjunct_break__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_indic_conjunct_break__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "indic_conjunct_break__chr_as_str_wrong"
# subject = "unicodedata.indic_conjunct_break(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.indic_conjunct_break(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import indic_conjunct_break
try:
    indic_conjunct_break(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/is_normalized__form_as__NormalizationForm_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_is_normalized__form_as__NormalizationForm_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "is_normalized__form_as__NormalizationForm_wrong"
# subject = "unicodedata.is_normalized(form: _NormalizationForm)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.is_normalized(form: _NormalizationForm); call it with the wrong type.

typeshed contract: form is _NormalizationForm. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unicodedata import is_normalized
try:
    is_normalized(_W(), "")  # form: _NormalizationForm <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/isxidcontinue__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_isxidcontinue__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "isxidcontinue__chr_as_str_wrong"
# subject = "unicodedata.isxidcontinue(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.isxidcontinue(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import isxidcontinue
try:
    isxidcontinue(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/isxidstart__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_isxidstart__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "isxidstart__chr_as_str_wrong"
# subject = "unicodedata.isxidstart(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.isxidstart(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import isxidstart
try:
    isxidstart(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/iter_graphemes__unistr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_iter_graphemes__unistr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "iter_graphemes__unistr_as_str_wrong"
# subject = "unicodedata.iter_graphemes(unistr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.iter_graphemes(unistr: str); call it with the wrong type.

typeshed contract: unistr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import iter_graphemes
try:
    iter_graphemes(12345)  # unistr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/lookup__name_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_lookup__name_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "lookup__name_as_typed_wrong"
# subject = "unicodedata.lookup(name: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.lookup(name: typed); call it with the wrong type.

typeshed contract: name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unicodedata import lookup
try:
    lookup(_W())  # name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/mirrored__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_mirrored__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "mirrored__chr_as_str_wrong"
# subject = "unicodedata.mirrored(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.mirrored(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import mirrored
try:
    mirrored(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/name__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_name__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "name__chr_as_str_wrong"
# subject = "unicodedata.name(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.name(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import name
try:
    name(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/normalize__form_as__NormalizationForm_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_normalize__form_as__NormalizationForm_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "normalize__form_as__NormalizationForm_wrong"
# subject = "unicodedata.normalize(form: _NormalizationForm)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.normalize(form: _NormalizationForm); call it with the wrong type.

typeshed contract: form is _NormalizationForm. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unicodedata import normalize
try:
    normalize(_W(), "")  # form: _NormalizationForm <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unicodedata/numeric__chr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unicodedata_numeric__chr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "numeric__chr_as_str_wrong"
# subject = "unicodedata.numeric(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.numeric(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import numeric
try:
    numeric(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
