use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/code/InteractiveConsole__init__locals_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_code_InteractiveConsole__init__locals_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "type"
# case = "InteractiveConsole__init__locals_as_typed_wrong"
# subject = "code.InteractiveConsole.__init__(locals: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/code.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: code.InteractiveConsole.__init__(locals: typed); call it with the wrong type.

typeshed contract: locals is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from code import InteractiveConsole
try:
    InteractiveConsole(_W())  # locals: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/code/InteractiveConsole__interact__banner_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_code_InteractiveConsole__interact__banner_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "type"
# case = "InteractiveConsole__interact__banner_as_typed_wrong"
# subject = "code.InteractiveConsole.interact(banner: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/code.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: code.InteractiveConsole.interact(banner: typed); call it with the wrong type.

typeshed contract: banner is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from code import InteractiveConsole
obj = object.__new__(InteractiveConsole)
try:
    obj.interact(_W())  # banner: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/code/InteractiveConsole__push__line_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_code_InteractiveConsole__push__line_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "type"
# case = "InteractiveConsole__push__line_as_str_wrong"
# subject = "code.InteractiveConsole.push(line: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/code.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: code.InteractiveConsole.push(line: str); call it with the wrong type.

typeshed contract: line is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from code import InteractiveConsole
obj = object.__new__(InteractiveConsole)
try:
    obj.push(12345)  # line: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/code/InteractiveConsole__raw_input__prompt_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_code_InteractiveConsole__raw_input__prompt_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "type"
# case = "InteractiveConsole__raw_input__prompt_as_str_wrong"
# subject = "code.InteractiveConsole.raw_input(prompt: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/code.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: code.InteractiveConsole.raw_input(prompt: str); call it with the wrong type.

typeshed contract: prompt is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from code import InteractiveConsole
obj = object.__new__(InteractiveConsole)
try:
    obj.raw_input(12345)  # prompt: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/code/InteractiveInterpreter__init__locals_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_code_InteractiveInterpreter__init__locals_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "type"
# case = "InteractiveInterpreter__init__locals_as_typed_wrong"
# subject = "code.InteractiveInterpreter.__init__(locals: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/code.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: code.InteractiveInterpreter.__init__(locals: typed); call it with the wrong type.

typeshed contract: locals is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from code import InteractiveInterpreter
try:
    InteractiveInterpreter(_W())  # locals: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/code/InteractiveInterpreter__runcode__code_as_CodeType_wrong.py`.
#[test]
fn test_gen_type_std_libs_code_InteractiveInterpreter__runcode__code_as_CodeType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "type"
# case = "InteractiveInterpreter__runcode__code_as_CodeType_wrong"
# subject = "code.InteractiveInterpreter.runcode(code: CodeType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/code.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: code.InteractiveInterpreter.runcode(code: CodeType); call it with the wrong type.

typeshed contract: code is CodeType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from code import InteractiveInterpreter
obj = object.__new__(InteractiveInterpreter)
try:
    obj.runcode(_W())  # code: CodeType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/code/InteractiveInterpreter__runsource__source_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_code_InteractiveInterpreter__runsource__source_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "type"
# case = "InteractiveInterpreter__runsource__source_as_str_wrong"
# subject = "code.InteractiveInterpreter.runsource(source: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/code.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: code.InteractiveInterpreter.runsource(source: str); call it with the wrong type.

typeshed contract: source is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from code import InteractiveInterpreter
obj = object.__new__(InteractiveInterpreter)
try:
    obj.runsource(12345)  # source: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/code/InteractiveInterpreter__showsyntaxerror__filename_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_code_InteractiveInterpreter__showsyntaxerror__filename_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "type"
# case = "InteractiveInterpreter__showsyntaxerror__filename_as_typed_wrong"
# subject = "code.InteractiveInterpreter.showsyntaxerror(filename: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/code.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: code.InteractiveInterpreter.showsyntaxerror(filename: typed); call it with the wrong type.

typeshed contract: filename is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from code import InteractiveInterpreter
obj = object.__new__(InteractiveInterpreter)
try:
    obj.showsyntaxerror(_W())  # filename: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/code/InteractiveInterpreter__write__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_code_InteractiveInterpreter__write__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "type"
# case = "InteractiveInterpreter__write__data_as_str_wrong"
# subject = "code.InteractiveInterpreter.write(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/code.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: code.InteractiveInterpreter.write(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from code import InteractiveInterpreter
obj = object.__new__(InteractiveInterpreter)
try:
    obj.write(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/code/interact__banner_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_code_interact__banner_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "type"
# case = "interact__banner_as_typed_wrong"
# subject = "code.interact(banner: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/code.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: code.interact(banner: typed); call it with the wrong type.

typeshed contract: banner is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from code import interact
try:
    interact(_W())  # banner: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
