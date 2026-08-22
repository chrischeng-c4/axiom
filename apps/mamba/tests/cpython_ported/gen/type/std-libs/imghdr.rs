use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/imghdr/what__file_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_imghdr_what__file_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imghdr"
# dimension = "type"
# case = "what__file_as_typed_wrong"
# subject = "imghdr.what(file: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/imghdr.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: imghdr.what(file: typed); call it with the wrong type.

typeshed contract: file is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from imghdr import what
try:
    what(_W())  # file: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/imghdr/what__h_as_bytes_wrong.py`.
#[test]
fn test_gen_type_std_libs_imghdr_what__h_as_bytes_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imghdr"
# dimension = "type"
# case = "what__h_as_bytes_wrong"
# subject = "imghdr.what(h: bytes)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/imghdr.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: imghdr.what(h: bytes); call it with the wrong type.

typeshed contract: h is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from imghdr import what
try:
    what(None, 12345)  # h: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
