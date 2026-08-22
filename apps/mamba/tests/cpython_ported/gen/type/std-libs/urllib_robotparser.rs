use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/urllib_robotparser/RobotFileParser__can_fetch__useragent_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_robotparser_RobotFileParser__can_fetch__useragent_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "type"
# case = "RobotFileParser__can_fetch__useragent_as_str_wrong"
# subject = "urllib.robotparser.RobotFileParser.can_fetch(useragent: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/robotparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.robotparser.RobotFileParser.can_fetch(useragent: str); call it with the wrong type.

typeshed contract: useragent is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.robotparser import RobotFileParser
obj = object.__new__(RobotFileParser)
try:
    obj.can_fetch(12345, "")  # useragent: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_robotparser/RobotFileParser__crawl_delay__useragent_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_robotparser_RobotFileParser__crawl_delay__useragent_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "type"
# case = "RobotFileParser__crawl_delay__useragent_as_str_wrong"
# subject = "urllib.robotparser.RobotFileParser.crawl_delay(useragent: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/robotparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.robotparser.RobotFileParser.crawl_delay(useragent: str); call it with the wrong type.

typeshed contract: useragent is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.robotparser import RobotFileParser
obj = object.__new__(RobotFileParser)
try:
    obj.crawl_delay(12345)  # useragent: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_robotparser/RobotFileParser__init__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_robotparser_RobotFileParser__init__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "type"
# case = "RobotFileParser__init__url_as_str_wrong"
# subject = "urllib.robotparser.RobotFileParser.__init__(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/robotparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.robotparser.RobotFileParser.__init__(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.robotparser import RobotFileParser
try:
    RobotFileParser(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_robotparser/RobotFileParser__parse__lines_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_robotparser_RobotFileParser__parse__lines_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "type"
# case = "RobotFileParser__parse__lines_as_Iterable_wrong"
# subject = "urllib.robotparser.RobotFileParser.parse(lines: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/robotparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.robotparser.RobotFileParser.parse(lines: Iterable); call it with the wrong type.

typeshed contract: lines is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.robotparser import RobotFileParser
obj = object.__new__(RobotFileParser)
try:
    obj.parse(_W())  # lines: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_robotparser/RobotFileParser__request_rate__useragent_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_robotparser_RobotFileParser__request_rate__useragent_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "type"
# case = "RobotFileParser__request_rate__useragent_as_str_wrong"
# subject = "urllib.robotparser.RobotFileParser.request_rate(useragent: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/robotparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.robotparser.RobotFileParser.request_rate(useragent: str); call it with the wrong type.

typeshed contract: useragent is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.robotparser import RobotFileParser
obj = object.__new__(RobotFileParser)
try:
    obj.request_rate(12345)  # useragent: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_robotparser/RobotFileParser__set_url__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_robotparser_RobotFileParser__set_url__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "type"
# case = "RobotFileParser__set_url__url_as_str_wrong"
# subject = "urllib.robotparser.RobotFileParser.set_url(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/robotparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.robotparser.RobotFileParser.set_url(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.robotparser import RobotFileParser
obj = object.__new__(RobotFileParser)
try:
    obj.set_url(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
