use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/crypt/crypt__word_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_crypt_crypt__word_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "crypt"
# dimension = "type"
# case = "crypt__word_as_str_wrong"
# subject = "crypt.crypt(word: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/crypt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: crypt.crypt(word: str); call it with the wrong type.

typeshed contract: word is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from crypt import crypt
try:
    crypt(12345)  # word: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/crypt/mksalt__method_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_crypt_mksalt__method_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "crypt"
# dimension = "type"
# case = "mksalt__method_as_typed_wrong"
# subject = "crypt.mksalt(method: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/crypt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: crypt.mksalt(method: typed); call it with the wrong type.

typeshed contract: method is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from crypt import mksalt
try:
    mksalt(_W())  # method: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
