use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/lib2to3_fixer_base/BaseFix__cannot_convert__node_as_Base_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_fixer_base_BaseFix__cannot_convert__node_as_Base_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "BaseFix__cannot_convert__node_as_Base_wrong"
# subject = "lib2to3.fixer_base.BaseFix.cannot_convert(node: Base)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.BaseFix.cannot_convert(node: Base); call it with the wrong type.

typeshed contract: node is Base. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.fixer_base import BaseFix
obj = object.__new__(BaseFix)
try:
    obj.cannot_convert(_W())  # node: Base <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_fixer_base/BaseFix__finish_tree__tree_as_Node_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_fixer_base_BaseFix__finish_tree__tree_as_Node_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "BaseFix__finish_tree__tree_as_Node_wrong"
# subject = "lib2to3.fixer_base.BaseFix.finish_tree(tree: Node)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.BaseFix.finish_tree(tree: Node); call it with the wrong type.

typeshed contract: tree is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.fixer_base import BaseFix
obj = object.__new__(BaseFix)
try:
    obj.finish_tree(_W(), None)  # tree: Node <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_fixer_base/BaseFix__init__options_as_MutableMapping_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_fixer_base_BaseFix__init__options_as_MutableMapping_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "BaseFix__init__options_as_MutableMapping_wrong"
# subject = "lib2to3.fixer_base.BaseFix.__init__(options: MutableMapping)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.BaseFix.__init__(options: MutableMapping); call it with the wrong type.

typeshed contract: options is MutableMapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.fixer_base import BaseFix
try:
    BaseFix(_W(), None)  # options: MutableMapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_fixer_base/BaseFix__log_message__message_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_fixer_base_BaseFix__log_message__message_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "BaseFix__log_message__message_as_str_wrong"
# subject = "lib2to3.fixer_base.BaseFix.log_message(message: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.BaseFix.log_message(message: str); call it with the wrong type.

typeshed contract: message is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.fixer_base import BaseFix
obj = object.__new__(BaseFix)
try:
    obj.log_message(12345)  # message: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_fixer_base/BaseFix__match__node_as__N_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_fixer_base_BaseFix__match__node_as__N_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "BaseFix__match__node_as__N_wrong"
# subject = "lib2to3.fixer_base.BaseFix.match(node: _N)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.BaseFix.match(node: _N); call it with the wrong type.

typeshed contract: node is _N. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.fixer_base import BaseFix
obj = object.__new__(BaseFix)
try:
    obj.match(_W())  # node: _N <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_fixer_base/BaseFix__new_name__template_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_fixer_base_BaseFix__new_name__template_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "BaseFix__new_name__template_as_str_wrong"
# subject = "lib2to3.fixer_base.BaseFix.new_name(template: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.BaseFix.new_name(template: str); call it with the wrong type.

typeshed contract: template is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.fixer_base import BaseFix
obj = object.__new__(BaseFix)
try:
    obj.new_name(12345)  # template: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_fixer_base/BaseFix__set_filename__filename_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_fixer_base_BaseFix__set_filename__filename_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "BaseFix__set_filename__filename_as_StrPath_wrong"
# subject = "lib2to3.fixer_base.BaseFix.set_filename(filename: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.BaseFix.set_filename(filename: StrPath); call it with the wrong type.

typeshed contract: filename is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.fixer_base import BaseFix
obj = object.__new__(BaseFix)
try:
    obj.set_filename(_W())  # filename: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_fixer_base/BaseFix__start_tree__tree_as_Node_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_fixer_base_BaseFix__start_tree__tree_as_Node_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "BaseFix__start_tree__tree_as_Node_wrong"
# subject = "lib2to3.fixer_base.BaseFix.start_tree(tree: Node)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.BaseFix.start_tree(tree: Node); call it with the wrong type.

typeshed contract: tree is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.fixer_base import BaseFix
obj = object.__new__(BaseFix)
try:
    obj.start_tree(_W(), None)  # tree: Node <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_fixer_base/BaseFix__transform__node_as_Base_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_fixer_base_BaseFix__transform__node_as_Base_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "BaseFix__transform__node_as_Base_wrong"
# subject = "lib2to3.fixer_base.BaseFix.transform(node: Base)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.BaseFix.transform(node: Base); call it with the wrong type.

typeshed contract: node is Base. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.fixer_base import BaseFix
obj = object.__new__(BaseFix)
try:
    obj.transform(_W(), None)  # node: Base <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_fixer_base/BaseFix__warning__node_as_Base_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_fixer_base_BaseFix__warning__node_as_Base_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "BaseFix__warning__node_as_Base_wrong"
# subject = "lib2to3.fixer_base.BaseFix.warning(node: Base)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.BaseFix.warning(node: Base); call it with the wrong type.

typeshed contract: node is Base. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.fixer_base import BaseFix
obj = object.__new__(BaseFix)
try:
    obj.warning(_W(), "")  # node: Base <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_fixer_base/ConditionalFix__should_skip__node_as_Base_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_fixer_base_ConditionalFix__should_skip__node_as_Base_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "ConditionalFix__should_skip__node_as_Base_wrong"
# subject = "lib2to3.fixer_base.ConditionalFix.should_skip(node: Base)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.ConditionalFix.should_skip(node: Base); call it with the wrong type.

typeshed contract: node is Base. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.fixer_base import ConditionalFix
obj = object.__new__(ConditionalFix)
try:
    obj.should_skip(_W())  # node: Base <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_fixer_base/ConditionalFix__start_tree__tree_as_Node_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_fixer_base_ConditionalFix__start_tree__tree_as_Node_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "ConditionalFix__start_tree__tree_as_Node_wrong"
# subject = "lib2to3.fixer_base.ConditionalFix.start_tree(tree: Node)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.ConditionalFix.start_tree(tree: Node); call it with the wrong type.

typeshed contract: tree is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.fixer_base import ConditionalFix
obj = object.__new__(ConditionalFix)
try:
    obj.start_tree(_W(), None)  # tree: Node <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
