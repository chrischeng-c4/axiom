use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/email_iterators/body_line_iterator__msg_as_Message_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_iterators_body_line_iterator__msg_as_Message_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_iterators"
# dimension = "type"
# case = "body_line_iterator__msg_as_Message_wrong"
# subject = "email.iterators.body_line_iterator(msg: Message)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/iterators.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.iterators.body_line_iterator(msg: Message); call it with the wrong type.

typeshed contract: msg is Message. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.iterators import body_line_iterator
try:
    body_line_iterator(_W())  # msg: Message <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_iterators/typed_subpart_iterator__msg_as_Message_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_iterators_typed_subpart_iterator__msg_as_Message_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_iterators"
# dimension = "type"
# case = "typed_subpart_iterator__msg_as_Message_wrong"
# subject = "email.iterators.typed_subpart_iterator(msg: Message)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/iterators.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.iterators.typed_subpart_iterator(msg: Message); call it with the wrong type.

typeshed contract: msg is Message. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.iterators import typed_subpart_iterator
try:
    typed_subpart_iterator(_W())  # msg: Message <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_iterators/walk__self_as_Message_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_iterators_walk__self_as_Message_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_iterators"
# dimension = "type"
# case = "walk__self_as_Message_wrong"
# subject = "email.iterators.walk(self: Message)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/iterators.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.iterators.walk(self: Message); call it with the wrong type.

typeshed contract: self is Message. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.iterators import walk
try:
    walk(_W())  # self: Message <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
