use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/email_policy/EmailPolicy__fold__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_policy_EmailPolicy__fold__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_policy"
# dimension = "type"
# case = "EmailPolicy__fold__name_as_str_wrong"
# subject = "email.policy.EmailPolicy.fold(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/policy.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.policy.EmailPolicy.fold(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.policy import EmailPolicy
obj = object.__new__(EmailPolicy)
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

/// Ported from `tests/cpython/type/std-libs/email_policy/EmailPolicy__fold_binary__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_policy_EmailPolicy__fold_binary__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_policy"
# dimension = "type"
# case = "EmailPolicy__fold_binary__name_as_str_wrong"
# subject = "email.policy.EmailPolicy.fold_binary(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/policy.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.policy.EmailPolicy.fold_binary(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.policy import EmailPolicy
obj = object.__new__(EmailPolicy)
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

/// Ported from `tests/cpython/type/std-libs/email_policy/EmailPolicy__header_fetch_parse__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_policy_EmailPolicy__header_fetch_parse__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_policy"
# dimension = "type"
# case = "EmailPolicy__header_fetch_parse__name_as_str_wrong"
# subject = "email.policy.EmailPolicy.header_fetch_parse(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/policy.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.policy.EmailPolicy.header_fetch_parse(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.policy import EmailPolicy
obj = object.__new__(EmailPolicy)
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

/// Ported from `tests/cpython/type/std-libs/email_policy/EmailPolicy__header_source_parse__sourcelines_as_list_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_policy_EmailPolicy__header_source_parse__sourcelines_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_policy"
# dimension = "type"
# case = "EmailPolicy__header_source_parse__sourcelines_as_list_wrong"
# subject = "email.policy.EmailPolicy.header_source_parse(sourcelines: list)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/policy.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.policy.EmailPolicy.header_source_parse(sourcelines: list); call it with the wrong type.

typeshed contract: sourcelines is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.policy import EmailPolicy
obj = object.__new__(EmailPolicy)
try:
    obj.header_source_parse(12345)  # sourcelines: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_policy/EmailPolicy__header_store_parse__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_policy_EmailPolicy__header_store_parse__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_policy"
# dimension = "type"
# case = "EmailPolicy__header_store_parse__name_as_str_wrong"
# subject = "email.policy.EmailPolicy.header_store_parse(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/policy.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.policy.EmailPolicy.header_store_parse(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.policy import EmailPolicy
obj = object.__new__(EmailPolicy)
try:
    obj.header_store_parse(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
