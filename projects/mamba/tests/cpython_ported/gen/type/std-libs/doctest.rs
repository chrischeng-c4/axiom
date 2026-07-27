use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/doctest/DocFileTest__path_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocFileTest__path_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocFileTest__path_as_str_wrong"
# subject = "doctest.DocFileTest(path: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocFileTest(path: str); call it with the wrong type.

typeshed contract: path is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import DocFileTest
try:
    DocFileTest(12345)  # path: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestCase__format_failure__err_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestCase__format_failure__err_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestCase__format_failure__err_as_str_wrong"
# subject = "doctest.DocTestCase.format_failure(err: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestCase.format_failure(err: str); call it with the wrong type.

typeshed contract: err is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import DocTestCase
obj = object.__new__(DocTestCase)
try:
    obj.format_failure(12345)  # err: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestCase__init__test_as_DocTest_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestCase__init__test_as_DocTest_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestCase__init__test_as_DocTest_wrong"
# subject = "doctest.DocTestCase.__init__(test: DocTest)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestCase.__init__(test: DocTest); call it with the wrong type.

typeshed contract: test is DocTest. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTestCase
try:
    DocTestCase(_W())  # test: DocTest <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestFailure__init__test_as_DocTest_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestFailure__init__test_as_DocTest_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestFailure__init__test_as_DocTest_wrong"
# subject = "doctest.DocTestFailure.__init__(test: DocTest)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestFailure.__init__(test: DocTest); call it with the wrong type.

typeshed contract: test is DocTest. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTestFailure
try:
    DocTestFailure(_W(), None, "")  # test: DocTest <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestFinder__find__name_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestFinder__find__name_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestFinder__find__name_as_typed_wrong"
# subject = "doctest.DocTestFinder.find(name: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestFinder.find(name: typed); call it with the wrong type.

typeshed contract: name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTestFinder
obj = object.__new__(DocTestFinder)
try:
    obj.find(None, _W())  # name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestFinder__init__verbose_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestFinder__init__verbose_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestFinder__init__verbose_as_bool_wrong"
# subject = "doctest.DocTestFinder.__init__(verbose: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestFinder.__init__(verbose: bool); call it with the wrong type.

typeshed contract: verbose is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import DocTestFinder
try:
    DocTestFinder("not_a_bool")  # verbose: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestParser__get_doctest__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestParser__get_doctest__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestParser__get_doctest__string_as_str_wrong"
# subject = "doctest.DocTestParser.get_doctest(string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestParser.get_doctest(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import DocTestParser
obj = object.__new__(DocTestParser)
try:
    obj.get_doctest(12345, None, "", None, None)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestParser__get_examples__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestParser__get_examples__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestParser__get_examples__string_as_str_wrong"
# subject = "doctest.DocTestParser.get_examples(string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestParser.get_examples(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import DocTestParser
obj = object.__new__(DocTestParser)
try:
    obj.get_examples(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestParser__parse__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestParser__parse__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestParser__parse__string_as_str_wrong"
# subject = "doctest.DocTestParser.parse(string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestParser.parse(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import DocTestParser
obj = object.__new__(DocTestParser)
try:
    obj.parse(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestRunner__init__checker_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestRunner__init__checker_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestRunner__init__checker_as_typed_wrong"
# subject = "doctest.DocTestRunner.__init__(checker: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestRunner.__init__(checker: typed); call it with the wrong type.

typeshed contract: checker is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTestRunner
try:
    DocTestRunner(_W())  # checker: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestRunner__merge__other_as_DocTestRunner_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestRunner__merge__other_as_DocTestRunner_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestRunner__merge__other_as_DocTestRunner_wrong"
# subject = "doctest.DocTestRunner.merge(other: DocTestRunner)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestRunner.merge(other: DocTestRunner); call it with the wrong type.

typeshed contract: other is DocTestRunner. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTestRunner
obj = object.__new__(DocTestRunner)
try:
    obj.merge(_W())  # other: DocTestRunner <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestRunner__report_failure__out_as__Out_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestRunner__report_failure__out_as__Out_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestRunner__report_failure__out_as__Out_wrong"
# subject = "doctest.DocTestRunner.report_failure(out: _Out)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestRunner.report_failure(out: _Out); call it with the wrong type.

typeshed contract: out is _Out. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTestRunner
obj = object.__new__(DocTestRunner)
try:
    obj.report_failure(_W(), None, None, "")  # out: _Out <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestRunner__report_start__out_as__Out_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestRunner__report_start__out_as__Out_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestRunner__report_start__out_as__Out_wrong"
# subject = "doctest.DocTestRunner.report_start(out: _Out)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestRunner.report_start(out: _Out); call it with the wrong type.

typeshed contract: out is _Out. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTestRunner
obj = object.__new__(DocTestRunner)
try:
    obj.report_start(_W(), None, None)  # out: _Out <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestRunner__report_success__out_as__Out_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestRunner__report_success__out_as__Out_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestRunner__report_success__out_as__Out_wrong"
# subject = "doctest.DocTestRunner.report_success(out: _Out)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestRunner.report_success(out: _Out); call it with the wrong type.

typeshed contract: out is _Out. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTestRunner
obj = object.__new__(DocTestRunner)
try:
    obj.report_success(_W(), None, None, "")  # out: _Out <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestRunner__report_unexpected_exception__out_as__Out_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestRunner__report_unexpected_exception__out_as__Out_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestRunner__report_unexpected_exception__out_as__Out_wrong"
# subject = "doctest.DocTestRunner.report_unexpected_exception(out: _Out)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestRunner.report_unexpected_exception(out: _Out); call it with the wrong type.

typeshed contract: out is _Out. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTestRunner
obj = object.__new__(DocTestRunner)
try:
    obj.report_unexpected_exception(_W(), None, None, None)  # out: _Out <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestRunner__run__test_as_DocTest_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestRunner__run__test_as_DocTest_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestRunner__run__test_as_DocTest_wrong"
# subject = "doctest.DocTestRunner.run(test: DocTest)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestRunner.run(test: DocTest); call it with the wrong type.

typeshed contract: test is DocTest. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTestRunner
obj = object.__new__(DocTestRunner)
try:
    obj.run(_W())  # test: DocTest <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestRunner__summarize__verbose_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestRunner__summarize__verbose_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestRunner__summarize__verbose_as_typed_wrong"
# subject = "doctest.DocTestRunner.summarize(verbose: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestRunner.summarize(verbose: typed); call it with the wrong type.

typeshed contract: verbose is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTestRunner
obj = object.__new__(DocTestRunner)
try:
    obj.summarize(_W())  # verbose: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTestSuite__module_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTestSuite__module_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTestSuite__module_as_typed_wrong"
# subject = "doctest.DocTestSuite(module: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTestSuite(module: typed); call it with the wrong type.

typeshed contract: module is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTestSuite
try:
    DocTestSuite(_W())  # module: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTest____lt____other_as_DocTest_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTest____lt____other_as_DocTest_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTest____lt____other_as_DocTest_wrong"
# subject = "doctest.DocTest.__lt__(other: DocTest)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTest.__lt__(other: DocTest); call it with the wrong type.

typeshed contract: other is DocTest. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import DocTest
obj = object.__new__(DocTest)
try:
    obj.__lt__(_W())  # other: DocTest <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/DocTest__init__examples_as_list_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_DocTest__init__examples_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "DocTest__init__examples_as_list_wrong"
# subject = "doctest.DocTest.__init__(examples: list)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.DocTest.__init__(examples: list); call it with the wrong type.

typeshed contract: examples is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import DocTest
try:
    DocTest(12345, None, "", None, None, None)  # examples: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/Example__init__source_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_Example__init__source_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "Example__init__source_as_str_wrong"
# subject = "doctest.Example.__init__(source: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.Example.__init__(source: str); call it with the wrong type.

typeshed contract: source is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import Example
try:
    Example(12345, "")  # source: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/OutputChecker__check_output__want_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_OutputChecker__check_output__want_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "OutputChecker__check_output__want_as_str_wrong"
# subject = "doctest.OutputChecker.check_output(want: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.OutputChecker.check_output(want: str); call it with the wrong type.

typeshed contract: want is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import OutputChecker
obj = object.__new__(OutputChecker)
try:
    obj.check_output(12345, "", 0)  # want: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/OutputChecker__output_difference__example_as_Example_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_OutputChecker__output_difference__example_as_Example_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "OutputChecker__output_difference__example_as_Example_wrong"
# subject = "doctest.OutputChecker.output_difference(example: Example)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.OutputChecker.output_difference(example: Example); call it with the wrong type.

typeshed contract: example is Example. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import OutputChecker
obj = object.__new__(OutputChecker)
try:
    obj.output_difference(_W(), "", 0)  # example: Example <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/SkipDocTestCase__init__module_as_ModuleType_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_SkipDocTestCase__init__module_as_ModuleType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "SkipDocTestCase__init__module_as_ModuleType_wrong"
# subject = "doctest.SkipDocTestCase.__init__(module: ModuleType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.SkipDocTestCase.__init__(module: ModuleType); call it with the wrong type.

typeshed contract: module is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import SkipDocTestCase
try:
    SkipDocTestCase(_W())  # module: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/TestResults____new____failed_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_TestResults____new____failed_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "TestResults____new____failed_as_int_wrong"
# subject = "doctest.TestResults.__new__(failed: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.TestResults.__new__(failed: int); call it with the wrong type.

typeshed contract: failed is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import TestResults
obj = object.__new__(TestResults)
try:
    obj.__new__("not_an_int", 0)  # failed: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/UnexpectedException__init__test_as_DocTest_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_UnexpectedException__init__test_as_DocTest_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "UnexpectedException__init__test_as_DocTest_wrong"
# subject = "doctest.UnexpectedException.__init__(test: DocTest)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.UnexpectedException.__init__(test: DocTest); call it with the wrong type.

typeshed contract: test is DocTest. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import UnexpectedException
try:
    UnexpectedException(_W(), None, None)  # test: DocTest <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/debug__module_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_debug__module_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "debug__module_as_typed_wrong"
# subject = "doctest.debug(module: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.debug(module: typed); call it with the wrong type.

typeshed contract: module is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import debug
try:
    debug(_W(), "")  # module: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/debug_script__src_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_debug_script__src_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "debug_script__src_as_str_wrong"
# subject = "doctest.debug_script(src: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.debug_script(src: str); call it with the wrong type.

typeshed contract: src is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import debug_script
try:
    debug_script(12345)  # src: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/debug_src__src_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_debug_src__src_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "debug_src__src_as_str_wrong"
# subject = "doctest.debug_src(src: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.debug_src(src: str); call it with the wrong type.

typeshed contract: src is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import debug_src
try:
    debug_src(12345)  # src: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/register_optionflag__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_register_optionflag__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "register_optionflag__name_as_str_wrong"
# subject = "doctest.register_optionflag(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.register_optionflag(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import register_optionflag
try:
    register_optionflag(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/run_docstring_examples__globs_as_dict_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_run_docstring_examples__globs_as_dict_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "run_docstring_examples__globs_as_dict_wrong"
# subject = "doctest.run_docstring_examples(globs: dict)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.run_docstring_examples(globs: dict); call it with the wrong type.

typeshed contract: globs is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import run_docstring_examples
try:
    run_docstring_examples(None, 12345)  # globs: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/script_from_examples__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_script_from_examples__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "script_from_examples__s_as_str_wrong"
# subject = "doctest.script_from_examples(s: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.script_from_examples(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import script_from_examples
try:
    script_from_examples(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/set_unittest_reportflags__flags_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_set_unittest_reportflags__flags_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "set_unittest_reportflags__flags_as_int_wrong"
# subject = "doctest.set_unittest_reportflags(flags: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.set_unittest_reportflags(flags: int); call it with the wrong type.

typeshed contract: flags is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import set_unittest_reportflags
try:
    set_unittest_reportflags("not_an_int")  # flags: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/testfile__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_testfile__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "testfile__filename_as_str_wrong"
# subject = "doctest.testfile(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.testfile(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from doctest import testfile
try:
    testfile(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/testmod__m_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_testmod__m_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "testmod__m_as_typed_wrong"
# subject = "doctest.testmod(m: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.testmod(m: typed); call it with the wrong type.

typeshed contract: m is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import testmod
try:
    testmod(_W())  # m: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/doctest/testsource__module_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_doctest_testsource__module_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "testsource__module_as_typed_wrong"
# subject = "doctest.testsource(module: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.testsource(module: typed); call it with the wrong type.

typeshed contract: module is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import testsource
try:
    testsource(_W(), "")  # module: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
