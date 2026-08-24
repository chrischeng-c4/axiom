use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/email_feedparser/BytesFeedParser__feed__data_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_feedparser_BytesFeedParser__feed__data_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_feedparser"
# dimension = "type"
# case = "BytesFeedParser__feed__data_as_typed_wrong"
# subject = "email.feedparser.BytesFeedParser.feed(data: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/feedparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.feedparser.BytesFeedParser.feed(data: typed); call it with the wrong type.

typeshed contract: data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.feedparser import BytesFeedParser
obj = object.__new__(BytesFeedParser)
try:
    obj.feed(_W())  # data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_feedparser/BytesFeedParser__init___factory_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_feedparser_BytesFeedParser__init___factory_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_feedparser"
# dimension = "type"
# case = "BytesFeedParser__init___factory_as_Callable_wrong"
# subject = "email.feedparser.BytesFeedParser.__init__(_factory: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/feedparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.feedparser.BytesFeedParser.__init__(_factory: Callable); call it with the wrong type.

typeshed contract: _factory is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.feedparser import BytesFeedParser
try:
    BytesFeedParser(_W())  # _factory: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_feedparser/BytesFeedParser__init___factory_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_feedparser_BytesFeedParser__init___factory_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_feedparser"
# dimension = "type"
# case = "BytesFeedParser__init___factory_as_typed_wrong"
# subject = "email.feedparser.BytesFeedParser.__init__(_factory: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/feedparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.feedparser.BytesFeedParser.__init__(_factory: typed); call it with the wrong type.

typeshed contract: _factory is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.feedparser import BytesFeedParser
try:
    BytesFeedParser(_W())  # _factory: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_feedparser/FeedParser__feed__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_feedparser_FeedParser__feed__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_feedparser"
# dimension = "type"
# case = "FeedParser__feed__data_as_str_wrong"
# subject = "email.feedparser.FeedParser.feed(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/feedparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.feedparser.FeedParser.feed(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.feedparser import FeedParser
obj = object.__new__(FeedParser)
try:
    obj.feed(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_feedparser/FeedParser__init___factory_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_feedparser_FeedParser__init___factory_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_feedparser"
# dimension = "type"
# case = "FeedParser__init___factory_as_Callable_wrong"
# subject = "email.feedparser.FeedParser.__init__(_factory: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/feedparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.feedparser.FeedParser.__init__(_factory: Callable); call it with the wrong type.

typeshed contract: _factory is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.feedparser import FeedParser
try:
    FeedParser(_W())  # _factory: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_feedparser/FeedParser__init___factory_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_feedparser_FeedParser__init___factory_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_feedparser"
# dimension = "type"
# case = "FeedParser__init___factory_as_typed_wrong"
# subject = "email.feedparser.FeedParser.__init__(_factory: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/feedparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.feedparser.FeedParser.__init__(_factory: typed); call it with the wrong type.

typeshed contract: _factory is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.feedparser import FeedParser
try:
    FeedParser(_W())  # _factory: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
