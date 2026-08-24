use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/tkinter_commondialog/Dialog__init__master_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_commondialog_Dialog__init__master_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_commondialog"
# dimension = "type"
# case = "Dialog__init__master_as_typed_wrong"
# subject = "tkinter.commondialog.Dialog.__init__(master: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/commondialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.commondialog.Dialog.__init__(master: typed); call it with the wrong type.

typeshed contract: master is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.commondialog import Dialog
try:
    Dialog(_W())  # master: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
