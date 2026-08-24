use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/email__policybase/Compat32__fold__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__policybase_Compat32__fold__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__policybase"
# dimension = "type"
# case = "Compat32__fold__name_as_str_wrong"
# subject = "email._policybase.Compat32.fold(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_policybase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._policybase.Compat32.fold(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._policybase import Compat32
obj = object.__new__(Compat32)
try:
    obj.fold(12345, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__policybase/Compat32__fold_binary__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__policybase_Compat32__fold_binary__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__policybase"
# dimension = "type"
# case = "Compat32__fold_binary__name_as_str_wrong"
# subject = "email._policybase.Compat32.fold_binary(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_policybase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._policybase.Compat32.fold_binary(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._policybase import Compat32
obj = object.__new__(Compat32)
try:
    obj.fold_binary(12345, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__policybase/Compat32__header_fetch_parse__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__policybase_Compat32__header_fetch_parse__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__policybase"
# dimension = "type"
# case = "Compat32__header_fetch_parse__name_as_str_wrong"
# subject = "email._policybase.Compat32.header_fetch_parse(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_policybase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._policybase.Compat32.header_fetch_parse(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._policybase import Compat32
obj = object.__new__(Compat32)
try:
    obj.header_fetch_parse(12345, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__policybase/Compat32__header_store_parse__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__policybase_Compat32__header_store_parse__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__policybase"
# dimension = "type"
# case = "Compat32__header_store_parse__name_as_str_wrong"
# subject = "email._policybase.Compat32.header_store_parse(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_policybase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._policybase.Compat32.header_store_parse(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._policybase import Compat32
obj = object.__new__(Compat32)
try:
    obj.header_store_parse(12345, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__policybase/Policy__fold__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__policybase_Policy__fold__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__policybase"
# dimension = "type"
# case = "Policy__fold__name_as_str_wrong"
# subject = "email._policybase.Policy.fold(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_policybase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._policybase.Policy.fold(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._policybase import Policy
obj = object.__new__(Policy)
try:
    obj.fold(12345, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__policybase/Policy__fold_binary__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__policybase_Policy__fold_binary__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__policybase"
# dimension = "type"
# case = "Policy__fold_binary__name_as_str_wrong"
# subject = "email._policybase.Policy.fold_binary(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_policybase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._policybase.Policy.fold_binary(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._policybase import Policy
obj = object.__new__(Policy)
try:
    obj.fold_binary(12345, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__policybase/Policy__header_fetch_parse__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__policybase_Policy__header_fetch_parse__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__policybase"
# dimension = "type"
# case = "Policy__header_fetch_parse__name_as_str_wrong"
# subject = "email._policybase.Policy.header_fetch_parse(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_policybase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._policybase.Policy.header_fetch_parse(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._policybase import Policy
obj = object.__new__(Policy)
try:
    obj.header_fetch_parse(12345, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__policybase/Policy__header_max_count__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__policybase_Policy__header_max_count__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__policybase"
# dimension = "type"
# case = "Policy__header_max_count__name_as_str_wrong"
# subject = "email._policybase.Policy.header_max_count(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_policybase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._policybase.Policy.header_max_count(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._policybase import Policy
obj = object.__new__(Policy)
try:
    obj.header_max_count(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__policybase/Policy__header_store_parse__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__policybase_Policy__header_store_parse__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__policybase"
# dimension = "type"
# case = "Policy__header_store_parse__name_as_str_wrong"
# subject = "email._policybase.Policy.header_store_parse(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_policybase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._policybase.Policy.header_store_parse(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._policybase import Policy
obj = object.__new__(Policy)
try:
    obj.header_store_parse(12345, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
