use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/email_errors/MessageDefect__init__line_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_errors_MessageDefect__init__line_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_errors"
# dimension = "type"
# case = "MessageDefect__init__line_as_typed_wrong"
# subject = "email.errors.MessageDefect.__init__(line: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/errors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.errors.MessageDefect.__init__(line: typed); call it with the wrong type.

typeshed contract: line is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.errors import MessageDefect
try:
    MessageDefect(_W())  # line: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_errors/NonPrintableDefect__init__non_printables_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_errors_NonPrintableDefect__init__non_printables_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_errors"
# dimension = "type"
# case = "NonPrintableDefect__init__non_printables_as_typed_wrong"
# subject = "email.errors.NonPrintableDefect.__init__(non_printables: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/errors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.errors.NonPrintableDefect.__init__(non_printables: typed); call it with the wrong type.

typeshed contract: non_printables is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.errors import NonPrintableDefect
try:
    NonPrintableDefect(_W())  # non_printables: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
