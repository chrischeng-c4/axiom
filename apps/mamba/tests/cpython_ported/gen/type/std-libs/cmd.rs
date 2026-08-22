use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/cmd/Cmd__columnize__list_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmd_Cmd__columnize__list_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "type"
# case = "Cmd__columnize__list_as_typed_wrong"
# subject = "cmd.Cmd.columnize(list: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmd.Cmd.columnize(list: typed); call it with the wrong type.

typeshed contract: list is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmd import Cmd
obj = object.__new__(Cmd)
try:
    obj.columnize(_W())  # list: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmd/Cmd__complete__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmd_Cmd__complete__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "type"
# case = "Cmd__complete__text_as_str_wrong"
# subject = "cmd.Cmd.complete(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmd.Cmd.complete(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cmd import Cmd
obj = object.__new__(Cmd)
try:
    obj.complete(12345, 0)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmd/Cmd__completenames__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmd_Cmd__completenames__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "type"
# case = "Cmd__completenames__text_as_str_wrong"
# subject = "cmd.Cmd.completenames(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmd.Cmd.completenames(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cmd import Cmd
obj = object.__new__(Cmd)
try:
    obj.completenames(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmd/Cmd__default__line_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmd_Cmd__default__line_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "type"
# case = "Cmd__default__line_as_str_wrong"
# subject = "cmd.Cmd.default(line: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmd.Cmd.default(line: str); call it with the wrong type.

typeshed contract: line is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cmd import Cmd
obj = object.__new__(Cmd)
try:
    obj.default(12345)  # line: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmd/Cmd__do_help__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmd_Cmd__do_help__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "type"
# case = "Cmd__do_help__arg_as_str_wrong"
# subject = "cmd.Cmd.do_help(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmd.Cmd.do_help(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cmd import Cmd
obj = object.__new__(Cmd)
try:
    obj.do_help(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmd/Cmd__init__completekey_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmd_Cmd__init__completekey_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "type"
# case = "Cmd__init__completekey_as_str_wrong"
# subject = "cmd.Cmd.__init__(completekey: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmd.Cmd.__init__(completekey: str); call it with the wrong type.

typeshed contract: completekey is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cmd import Cmd
try:
    Cmd(12345)  # completekey: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmd/Cmd__onecmd__line_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmd_Cmd__onecmd__line_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "type"
# case = "Cmd__onecmd__line_as_str_wrong"
# subject = "cmd.Cmd.onecmd(line: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmd.Cmd.onecmd(line: str); call it with the wrong type.

typeshed contract: line is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cmd import Cmd
obj = object.__new__(Cmd)
try:
    obj.onecmd(12345)  # line: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmd/Cmd__parseline__line_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmd_Cmd__parseline__line_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "type"
# case = "Cmd__parseline__line_as_str_wrong"
# subject = "cmd.Cmd.parseline(line: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmd.Cmd.parseline(line: str); call it with the wrong type.

typeshed contract: line is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cmd import Cmd
obj = object.__new__(Cmd)
try:
    obj.parseline(12345)  # line: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmd/Cmd__postcmd__stop_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmd_Cmd__postcmd__stop_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "type"
# case = "Cmd__postcmd__stop_as_bool_wrong"
# subject = "cmd.Cmd.postcmd(stop: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmd.Cmd.postcmd(stop: bool); call it with the wrong type.

typeshed contract: stop is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cmd import Cmd
obj = object.__new__(Cmd)
try:
    obj.postcmd("not_a_bool", "")  # stop: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmd/Cmd__precmd__line_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmd_Cmd__precmd__line_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "type"
# case = "Cmd__precmd__line_as_str_wrong"
# subject = "cmd.Cmd.precmd(line: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmd.Cmd.precmd(line: str); call it with the wrong type.

typeshed contract: line is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cmd import Cmd
obj = object.__new__(Cmd)
try:
    obj.precmd(12345)  # line: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmd/Cmd__print_topics__header_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmd_Cmd__print_topics__header_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "type"
# case = "Cmd__print_topics__header_as_str_wrong"
# subject = "cmd.Cmd.print_topics(header: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmd.Cmd.print_topics(header: str); call it with the wrong type.

typeshed contract: header is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cmd import Cmd
obj = object.__new__(Cmd)
try:
    obj.print_topics(12345, None, None, 0)  # header: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
