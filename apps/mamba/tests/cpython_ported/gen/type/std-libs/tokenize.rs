use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/tokenize/Untokenizer__add_backslash_continuation__start_as__Position_wrong.py`.
#[test]
fn test_gen_type_std_libs_tokenize_Untokenizer__add_backslash_continuation__start_as__Position_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "type"
# case = "Untokenizer__add_backslash_continuation__start_as__Position_wrong"
# subject = "tokenize.Untokenizer.add_backslash_continuation(start: _Position)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tokenize.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tokenize.Untokenizer.add_backslash_continuation(start: _Position); call it with the wrong type.

typeshed contract: start is _Position. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tokenize import Untokenizer
obj = object.__new__(Untokenizer)
try:
    obj.add_backslash_continuation(_W())  # start: _Position <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tokenize/Untokenizer__add_whitespace__start_as__Position_wrong.py`.
#[test]
fn test_gen_type_std_libs_tokenize_Untokenizer__add_whitespace__start_as__Position_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "type"
# case = "Untokenizer__add_whitespace__start_as__Position_wrong"
# subject = "tokenize.Untokenizer.add_whitespace(start: _Position)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tokenize.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tokenize.Untokenizer.add_whitespace(start: _Position); call it with the wrong type.

typeshed contract: start is _Position. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tokenize import Untokenizer
obj = object.__new__(Untokenizer)
try:
    obj.add_whitespace(_W())  # start: _Position <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tokenize/Untokenizer__escape_brackets__token_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_tokenize_Untokenizer__escape_brackets__token_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "type"
# case = "Untokenizer__escape_brackets__token_as_str_wrong"
# subject = "tokenize.Untokenizer.escape_brackets(token: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tokenize.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tokenize.Untokenizer.escape_brackets(token: str); call it with the wrong type.

typeshed contract: token is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tokenize import Untokenizer
obj = object.__new__(Untokenizer)
try:
    obj.escape_brackets(12345)  # token: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tokenize/Untokenizer__untokenize__iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_tokenize_Untokenizer__untokenize__iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "type"
# case = "Untokenizer__untokenize__iterable_as_Iterable_wrong"
# subject = "tokenize.Untokenizer.untokenize(iterable: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tokenize.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tokenize.Untokenizer.untokenize(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tokenize import Untokenizer
obj = object.__new__(Untokenizer)
try:
    obj.untokenize(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tokenize/open__filename_as_FileDescriptorOrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_tokenize_open__filename_as_FileDescriptorOrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "type"
# case = "open__filename_as_FileDescriptorOrPath_wrong"
# subject = "tokenize.open(filename: FileDescriptorOrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tokenize.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tokenize.open(filename: FileDescriptorOrPath); call it with the wrong type.

typeshed contract: filename is FileDescriptorOrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tokenize import open
try:
    open(_W())  # filename: FileDescriptorOrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tokenize/untokenize__iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_tokenize_untokenize__iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "type"
# case = "untokenize__iterable_as_Iterable_wrong"
# subject = "tokenize.untokenize(iterable: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tokenize.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tokenize.untokenize(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tokenize import untokenize
try:
    untokenize(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
