use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/secrets/choice__seq_as_SupportsLenAndGetItem_wrong.py`.
#[test]
fn test_gen_type_std_libs_secrets_choice__seq_as_SupportsLenAndGetItem_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "type"
# case = "choice__seq_as_SupportsLenAndGetItem_wrong"
# subject = "secrets.choice(seq: SupportsLenAndGetItem)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/secrets.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: secrets.choice(seq: SupportsLenAndGetItem); call it with the wrong type.

typeshed contract: seq is SupportsLenAndGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from secrets import choice
try:
    choice(_W())  # seq: SupportsLenAndGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/secrets/randbelow__exclusive_upper_bound_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_secrets_randbelow__exclusive_upper_bound_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "type"
# case = "randbelow__exclusive_upper_bound_as_int_wrong"
# subject = "secrets.randbelow(exclusive_upper_bound: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/secrets.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: secrets.randbelow(exclusive_upper_bound: int); call it with the wrong type.

typeshed contract: exclusive_upper_bound is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from secrets import randbelow
try:
    randbelow("not_an_int")  # exclusive_upper_bound: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/secrets/randbits__k_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_secrets_randbits__k_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "type"
# case = "randbits__k_as_int_wrong"
# subject = "secrets.randbits(k: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/secrets.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: secrets.randbits(k: int); call it with the wrong type.

typeshed contract: k is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from secrets import randbits
try:
    randbits("not_an_int")  # k: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/secrets/token_bytes__nbytes_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_secrets_token_bytes__nbytes_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "type"
# case = "token_bytes__nbytes_as_typed_wrong"
# subject = "secrets.token_bytes(nbytes: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/secrets.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: secrets.token_bytes(nbytes: typed); call it with the wrong type.

typeshed contract: nbytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from secrets import token_bytes
try:
    token_bytes(_W())  # nbytes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/secrets/token_hex__nbytes_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_secrets_token_hex__nbytes_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "type"
# case = "token_hex__nbytes_as_typed_wrong"
# subject = "secrets.token_hex(nbytes: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/secrets.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: secrets.token_hex(nbytes: typed); call it with the wrong type.

typeshed contract: nbytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from secrets import token_hex
try:
    token_hex(_W())  # nbytes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/secrets/token_urlsafe__nbytes_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_secrets_token_urlsafe__nbytes_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "type"
# case = "token_urlsafe__nbytes_as_typed_wrong"
# subject = "secrets.token_urlsafe(nbytes: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/secrets.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: secrets.token_urlsafe(nbytes: typed); call it with the wrong type.

typeshed contract: nbytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from secrets import token_urlsafe
try:
    token_urlsafe(_W())  # nbytes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
