use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/lib2to3_main/StdoutRefactoringTool__init__fixers_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_main_StdoutRefactoringTool__init__fixers_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_main"
# dimension = "type"
# case = "StdoutRefactoringTool__init__fixers_as_Iterable_wrong"
# subject = "lib2to3.main.StdoutRefactoringTool.__init__(fixers: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/main.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.main.StdoutRefactoringTool.__init__(fixers: Iterable); call it with the wrong type.

typeshed contract: fixers is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.main import StdoutRefactoringTool
try:
    StdoutRefactoringTool(_W(), None, None, True, True)  # fixers: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_main/StdoutRefactoringTool__log_error__msg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_main_StdoutRefactoringTool__log_error__msg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_main"
# dimension = "type"
# case = "StdoutRefactoringTool__log_error__msg_as_str_wrong"
# subject = "lib2to3.main.StdoutRefactoringTool.log_error(msg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/main.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.main.StdoutRefactoringTool.log_error(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.main import StdoutRefactoringTool
obj = object.__new__(StdoutRefactoringTool)
try:
    obj.log_error(12345)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_main/StdoutRefactoringTool__print_output__old_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_main_StdoutRefactoringTool__print_output__old_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_main"
# dimension = "type"
# case = "StdoutRefactoringTool__print_output__old_as_str_wrong"
# subject = "lib2to3.main.StdoutRefactoringTool.print_output(old: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/main.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.main.StdoutRefactoringTool.print_output(old: str); call it with the wrong type.

typeshed contract: old is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.main import StdoutRefactoringTool
obj = object.__new__(StdoutRefactoringTool)
try:
    obj.print_output(12345, "", "", True)  # old: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_main/StdoutRefactoringTool__write_file__new_text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_main_StdoutRefactoringTool__write_file__new_text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_main"
# dimension = "type"
# case = "StdoutRefactoringTool__write_file__new_text_as_str_wrong"
# subject = "lib2to3.main.StdoutRefactoringTool.write_file(new_text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/main.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.main.StdoutRefactoringTool.write_file(new_text: str); call it with the wrong type.

typeshed contract: new_text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.main import StdoutRefactoringTool
obj = object.__new__(StdoutRefactoringTool)
try:
    obj.write_file(12345, None, "", None)  # new_text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_main/diff_texts__a_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_main_diff_texts__a_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_main"
# dimension = "type"
# case = "diff_texts__a_as_str_wrong"
# subject = "lib2to3.main.diff_texts(a: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/main.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.main.diff_texts(a: str); call it with the wrong type.

typeshed contract: a is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.main import diff_texts
try:
    diff_texts(12345, "", "")  # a: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_main/main__fixer_pkg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_main_main__fixer_pkg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_main"
# dimension = "type"
# case = "main__fixer_pkg_as_str_wrong"
# subject = "lib2to3.main.main(fixer_pkg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/main.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.main.main(fixer_pkg: str); call it with the wrong type.

typeshed contract: fixer_pkg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.main import main
try:
    main(12345)  # fixer_pkg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
