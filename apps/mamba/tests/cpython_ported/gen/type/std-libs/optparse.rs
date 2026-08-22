use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/optparse/AmbiguousOptionError__init__opt_str_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_AmbiguousOptionError__init__opt_str_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "AmbiguousOptionError__init__opt_str_as_str_wrong"
# subject = "optparse.AmbiguousOptionError.__init__(opt_str: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.AmbiguousOptionError.__init__(opt_str: str); call it with the wrong type.

typeshed contract: opt_str is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import AmbiguousOptionError
try:
    AmbiguousOptionError(12345, None)  # opt_str: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/BadOptionError__init__opt_str_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_BadOptionError__init__opt_str_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "BadOptionError__init__opt_str_as_str_wrong"
# subject = "optparse.BadOptionError.__init__(opt_str: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.BadOptionError.__init__(opt_str: str); call it with the wrong type.

typeshed contract: opt_str is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import BadOptionError
try:
    BadOptionError(12345)  # opt_str: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/HelpFormatter__expand_default__option_as_Option_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_HelpFormatter__expand_default__option_as_Option_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__expand_default__option_as_Option_wrong"
# subject = "optparse.HelpFormatter.expand_default(option: Option)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.expand_default(option: Option); call it with the wrong type.

typeshed contract: option is Option. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.expand_default(_W())  # option: Option <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/HelpFormatter__format_description__description_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_HelpFormatter__format_description__description_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__format_description__description_as_typed_wrong"
# subject = "optparse.HelpFormatter.format_description(description: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.format_description(description: typed); call it with the wrong type.

typeshed contract: description is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.format_description(_W())  # description: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/HelpFormatter__format_epilog__epilog_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_HelpFormatter__format_epilog__epilog_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__format_epilog__epilog_as_typed_wrong"
# subject = "optparse.HelpFormatter.format_epilog(epilog: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.format_epilog(epilog: typed); call it with the wrong type.

typeshed contract: epilog is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.format_epilog(_W())  # epilog: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/HelpFormatter__format_heading__heading_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_HelpFormatter__format_heading__heading_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__format_heading__heading_as_str_wrong"
# subject = "optparse.HelpFormatter.format_heading(heading: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.format_heading(heading: str); call it with the wrong type.

typeshed contract: heading is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.format_heading(12345)  # heading: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/HelpFormatter__format_option__option_as_Option_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_HelpFormatter__format_option__option_as_Option_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__format_option__option_as_Option_wrong"
# subject = "optparse.HelpFormatter.format_option(option: Option)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.format_option(option: Option); call it with the wrong type.

typeshed contract: option is Option. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.format_option(_W())  # option: Option <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/HelpFormatter__format_option_strings__option_as_Option_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_HelpFormatter__format_option_strings__option_as_Option_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__format_option_strings__option_as_Option_wrong"
# subject = "optparse.HelpFormatter.format_option_strings(option: Option)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.format_option_strings(option: Option); call it with the wrong type.

typeshed contract: option is Option. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.format_option_strings(_W())  # option: Option <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/HelpFormatter__format_usage__usage_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_HelpFormatter__format_usage__usage_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__format_usage__usage_as_str_wrong"
# subject = "optparse.HelpFormatter.format_usage(usage: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.format_usage(usage: str); call it with the wrong type.

typeshed contract: usage is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.format_usage(12345)  # usage: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/HelpFormatter__init__indent_increment_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_HelpFormatter__init__indent_increment_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__init__indent_increment_as_int_wrong"
# subject = "optparse.HelpFormatter.__init__(indent_increment: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.__init__(indent_increment: int); call it with the wrong type.

typeshed contract: indent_increment is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import HelpFormatter
try:
    HelpFormatter("not_an_int", 0, None, None)  # indent_increment: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/HelpFormatter__set_long_opt_delimiter__delim_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_HelpFormatter__set_long_opt_delimiter__delim_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__set_long_opt_delimiter__delim_as_str_wrong"
# subject = "optparse.HelpFormatter.set_long_opt_delimiter(delim: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.set_long_opt_delimiter(delim: str); call it with the wrong type.

typeshed contract: delim is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.set_long_opt_delimiter(12345)  # delim: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/HelpFormatter__set_parser__parser_as_OptionParser_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_HelpFormatter__set_parser__parser_as_OptionParser_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__set_parser__parser_as_OptionParser_wrong"
# subject = "optparse.HelpFormatter.set_parser(parser: OptionParser)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.set_parser(parser: OptionParser); call it with the wrong type.

typeshed contract: parser is OptionParser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.set_parser(_W())  # parser: OptionParser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/HelpFormatter__set_short_opt_delimiter__delim_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_HelpFormatter__set_short_opt_delimiter__delim_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__set_short_opt_delimiter__delim_as_str_wrong"
# subject = "optparse.HelpFormatter.set_short_opt_delimiter(delim: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.set_short_opt_delimiter(delim: str); call it with the wrong type.

typeshed contract: delim is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.set_short_opt_delimiter(12345)  # delim: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/HelpFormatter__store_option_strings__parser_as_OptionParser_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_HelpFormatter__store_option_strings__parser_as_OptionParser_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__store_option_strings__parser_as_OptionParser_wrong"
# subject = "optparse.HelpFormatter.store_option_strings(parser: OptionParser)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.store_option_strings(parser: OptionParser); call it with the wrong type.

typeshed contract: parser is OptionParser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.store_option_strings(_W())  # parser: OptionParser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/IndentedHelpFormatter__format_heading__heading_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_IndentedHelpFormatter__format_heading__heading_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "IndentedHelpFormatter__format_heading__heading_as_str_wrong"
# subject = "optparse.IndentedHelpFormatter.format_heading(heading: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.IndentedHelpFormatter.format_heading(heading: str); call it with the wrong type.

typeshed contract: heading is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import IndentedHelpFormatter
obj = object.__new__(IndentedHelpFormatter)
try:
    obj.format_heading(12345)  # heading: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/IndentedHelpFormatter__format_usage__usage_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_IndentedHelpFormatter__format_usage__usage_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "IndentedHelpFormatter__format_usage__usage_as_str_wrong"
# subject = "optparse.IndentedHelpFormatter.format_usage(usage: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.IndentedHelpFormatter.format_usage(usage: str); call it with the wrong type.

typeshed contract: usage is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import IndentedHelpFormatter
obj = object.__new__(IndentedHelpFormatter)
try:
    obj.format_usage(12345)  # usage: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/IndentedHelpFormatter__init__indent_increment_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_IndentedHelpFormatter__init__indent_increment_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "IndentedHelpFormatter__init__indent_increment_as_int_wrong"
# subject = "optparse.IndentedHelpFormatter.__init__(indent_increment: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.IndentedHelpFormatter.__init__(indent_increment: int); call it with the wrong type.

typeshed contract: indent_increment is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import IndentedHelpFormatter
try:
    IndentedHelpFormatter("not_an_int")  # indent_increment: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptParseError__init__msg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptParseError__init__msg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptParseError__init__msg_as_str_wrong"
# subject = "optparse.OptParseError.__init__(msg: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptParseError.__init__(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptParseError
try:
    OptParseError(12345)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionContainer__add_option__opt_as_Option_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionContainer__add_option__opt_as_Option_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__add_option__opt_as_Option_wrong"
# subject = "optparse.OptionContainer.add_option(opt: Option)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.add_option(opt: Option); call it with the wrong type.

typeshed contract: opt is Option. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.add_option(_W())  # opt: Option <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionContainer__add_option__opt_str_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionContainer__add_option__opt_str_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__add_option__opt_str_as_str_wrong"
# subject = "optparse.OptionContainer.add_option(opt_str: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.add_option(opt_str: str); call it with the wrong type.

typeshed contract: opt_str is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.add_option(12345)  # opt_str: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionContainer__add_options__option_list_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionContainer__add_options__option_list_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__add_options__option_list_as_Iterable_wrong"
# subject = "optparse.OptionContainer.add_options(option_list: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.add_options(option_list: Iterable); call it with the wrong type.

typeshed contract: option_list is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.add_options(_W())  # option_list: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionContainer__format_description__formatter_as_HelpFormatter_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionContainer__format_description__formatter_as_HelpFormatter_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__format_description__formatter_as_HelpFormatter_wrong"
# subject = "optparse.OptionContainer.format_description(formatter: HelpFormatter)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.format_description(formatter: HelpFormatter); call it with the wrong type.

typeshed contract: formatter is HelpFormatter. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.format_description(_W())  # formatter: HelpFormatter <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionContainer__format_help__formatter_as_HelpFormatter_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionContainer__format_help__formatter_as_HelpFormatter_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__format_help__formatter_as_HelpFormatter_wrong"
# subject = "optparse.OptionContainer.format_help(formatter: HelpFormatter)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.format_help(formatter: HelpFormatter); call it with the wrong type.

typeshed contract: formatter is HelpFormatter. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.format_help(_W())  # formatter: HelpFormatter <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionContainer__format_option_help__formatter_as_HelpFormatter_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionContainer__format_option_help__formatter_as_HelpFormatter_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__format_option_help__formatter_as_HelpFormatter_wrong"
# subject = "optparse.OptionContainer.format_option_help(formatter: HelpFormatter)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.format_option_help(formatter: HelpFormatter); call it with the wrong type.

typeshed contract: formatter is HelpFormatter. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.format_option_help(_W())  # formatter: HelpFormatter <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionContainer__get_option__opt_str_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionContainer__get_option__opt_str_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__get_option__opt_str_as_str_wrong"
# subject = "optparse.OptionContainer.get_option(opt_str: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.get_option(opt_str: str); call it with the wrong type.

typeshed contract: opt_str is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.get_option(12345)  # opt_str: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionContainer__has_option__opt_str_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionContainer__has_option__opt_str_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__has_option__opt_str_as_str_wrong"
# subject = "optparse.OptionContainer.has_option(opt_str: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.has_option(opt_str: str); call it with the wrong type.

typeshed contract: opt_str is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.has_option(12345)  # opt_str: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionContainer__init__option_class_as_type_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionContainer__init__option_class_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__init__option_class_as_type_wrong"
# subject = "optparse.OptionContainer.__init__(option_class: type)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.__init__(option_class: type); call it with the wrong type.

typeshed contract: option_class is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionContainer
try:
    OptionContainer(_W(), None, None)  # option_class: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionContainer__remove_option__opt_str_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionContainer__remove_option__opt_str_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__remove_option__opt_str_as_str_wrong"
# subject = "optparse.OptionContainer.remove_option(opt_str: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.remove_option(opt_str: str); call it with the wrong type.

typeshed contract: opt_str is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.remove_option(12345)  # opt_str: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionContainer__set_conflict_handler__handler_as_Literal_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionContainer__set_conflict_handler__handler_as_Literal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__set_conflict_handler__handler_as_Literal_wrong"
# subject = "optparse.OptionContainer.set_conflict_handler(handler: Literal)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.set_conflict_handler(handler: Literal); call it with the wrong type.

typeshed contract: handler is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.set_conflict_handler(_W())  # handler: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionContainer__set_description__description_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionContainer__set_description__description_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__set_description__description_as_typed_wrong"
# subject = "optparse.OptionContainer.set_description(description: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.set_description(description: typed); call it with the wrong type.

typeshed contract: description is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.set_description(_W())  # description: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionError__init__msg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionError__init__msg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionError__init__msg_as_str_wrong"
# subject = "optparse.OptionError.__init__(msg: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionError.__init__(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionError
try:
    OptionError(12345, None)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionGroup__init__parser_as_OptionParser_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionGroup__init__parser_as_OptionParser_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionGroup__init__parser_as_OptionParser_wrong"
# subject = "optparse.OptionGroup.__init__(parser: OptionParser)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionGroup.__init__(parser: OptionParser); call it with the wrong type.

typeshed contract: parser is OptionParser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionGroup
try:
    OptionGroup(_W(), "")  # parser: OptionParser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionGroup__set_title__title_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionGroup__set_title__title_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionGroup__set_title__title_as_str_wrong"
# subject = "optparse.OptionGroup.set_title(title: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionGroup.set_title(title: str); call it with the wrong type.

typeshed contract: title is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionGroup
obj = object.__new__(OptionGroup)
try:
    obj.set_title(12345)  # title: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__add_option_group__opt_group_as_OptionGroup_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__add_option_group__opt_group_as_OptionGroup_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__add_option_group__opt_group_as_OptionGroup_wrong"
# subject = "optparse.OptionParser.add_option_group(opt_group: OptionGroup)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.add_option_group(opt_group: OptionGroup); call it with the wrong type.

typeshed contract: opt_group is OptionGroup. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.add_option_group(_W())  # opt_group: OptionGroup <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__add_option_group__title_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__add_option_group__title_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__add_option_group__title_as_str_wrong"
# subject = "optparse.OptionParser.add_option_group(title: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.add_option_group(title: str); call it with the wrong type.

typeshed contract: title is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.add_option_group(12345)  # title: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__check_values__values_as_Values_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__check_values__values_as_Values_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__check_values__values_as_Values_wrong"
# subject = "optparse.OptionParser.check_values(values: Values)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.check_values(values: Values); call it with the wrong type.

typeshed contract: values is Values. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.check_values(_W(), None)  # values: Values <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__error__msg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__error__msg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__error__msg_as_str_wrong"
# subject = "optparse.OptionParser.error(msg: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.error(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.error(12345)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__exit__status_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__exit__status_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__exit__status_as_int_wrong"
# subject = "optparse.OptionParser.exit(status: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.exit(status: int); call it with the wrong type.

typeshed contract: status is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.exit("not_an_int")  # status: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__expand_prog_name__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__expand_prog_name__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__expand_prog_name__s_as_str_wrong"
# subject = "optparse.OptionParser.expand_prog_name(s: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.expand_prog_name(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.expand_prog_name(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__format_epilog__formatter_as_HelpFormatter_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__format_epilog__formatter_as_HelpFormatter_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__format_epilog__formatter_as_HelpFormatter_wrong"
# subject = "optparse.OptionParser.format_epilog(formatter: HelpFormatter)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.format_epilog(formatter: HelpFormatter); call it with the wrong type.

typeshed contract: formatter is HelpFormatter. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.format_epilog(_W())  # formatter: HelpFormatter <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__format_help__formatter_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__format_help__formatter_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__format_help__formatter_as_typed_wrong"
# subject = "optparse.OptionParser.format_help(formatter: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.format_help(formatter: typed); call it with the wrong type.

typeshed contract: formatter is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.format_help(_W())  # formatter: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__format_option_help__formatter_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__format_option_help__formatter_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__format_option_help__formatter_as_typed_wrong"
# subject = "optparse.OptionParser.format_option_help(formatter: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.format_option_help(formatter: typed); call it with the wrong type.

typeshed contract: formatter is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.format_option_help(_W())  # formatter: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__get_option_group__opt_str_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__get_option_group__opt_str_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__get_option_group__opt_str_as_str_wrong"
# subject = "optparse.OptionParser.get_option_group(opt_str: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.get_option_group(opt_str: str); call it with the wrong type.

typeshed contract: opt_str is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.get_option_group(12345)  # opt_str: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__init__usage_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__init__usage_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__init__usage_as_typed_wrong"
# subject = "optparse.OptionParser.__init__(usage: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.__init__(usage: typed); call it with the wrong type.

typeshed contract: usage is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionParser
try:
    OptionParser(_W())  # usage: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__parse_args__args_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__parse_args__args_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__parse_args__args_as_typed_wrong"
# subject = "optparse.OptionParser.parse_args(args: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.parse_args(args: typed); call it with the wrong type.

typeshed contract: args is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.parse_args(_W())  # args: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__print_help__file_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__print_help__file_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__print_help__file_as_typed_wrong"
# subject = "optparse.OptionParser.print_help(file: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.print_help(file: typed); call it with the wrong type.

typeshed contract: file is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.print_help(_W())  # file: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__print_usage__file_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__print_usage__file_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__print_usage__file_as_typed_wrong"
# subject = "optparse.OptionParser.print_usage(file: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.print_usage(file: typed); call it with the wrong type.

typeshed contract: file is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.print_usage(_W())  # file: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__print_version__file_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__print_version__file_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__print_version__file_as_typed_wrong"
# subject = "optparse.OptionParser.print_version(file: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.print_version(file: typed); call it with the wrong type.

typeshed contract: file is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.print_version(_W())  # file: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__set_default__dest_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__set_default__dest_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__set_default__dest_as_str_wrong"
# subject = "optparse.OptionParser.set_default(dest: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.set_default(dest: str); call it with the wrong type.

typeshed contract: dest is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.set_default(12345, None)  # dest: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__set_process_default_values__process_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__set_process_default_values__process_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__set_process_default_values__process_as_bool_wrong"
# subject = "optparse.OptionParser.set_process_default_values(process: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.set_process_default_values(process: bool); call it with the wrong type.

typeshed contract: process is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.set_process_default_values("not_a_bool")  # process: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/OptionParser__set_usage__usage_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_OptionParser__set_usage__usage_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__set_usage__usage_as_typed_wrong"
# subject = "optparse.OptionParser.set_usage(usage: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.set_usage(usage: typed); call it with the wrong type.

typeshed contract: usage is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.set_usage(_W())  # usage: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/Option__check_value__opt_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_Option__check_value__opt_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "Option__check_value__opt_as_str_wrong"
# subject = "optparse.Option.check_value(opt: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.Option.check_value(opt: str); call it with the wrong type.

typeshed contract: opt is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import Option
obj = object.__new__(Option)
try:
    obj.check_value(12345, "")  # opt: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/Option__convert_value__opt_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_Option__convert_value__opt_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "Option__convert_value__opt_as_str_wrong"
# subject = "optparse.Option.convert_value(opt: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.Option.convert_value(opt: str); call it with the wrong type.

typeshed contract: opt is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import Option
obj = object.__new__(Option)
try:
    obj.convert_value(12345, None)  # opt: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/Option__process__opt_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_Option__process__opt_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "Option__process__opt_as_str_wrong"
# subject = "optparse.Option.process(opt: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.Option.process(opt: str); call it with the wrong type.

typeshed contract: opt is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import Option
obj = object.__new__(Option)
try:
    obj.process(12345, None, None, None)  # opt: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/Option__take_action__action_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_Option__take_action__action_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "Option__take_action__action_as_str_wrong"
# subject = "optparse.Option.take_action(action: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.Option.take_action(action: str); call it with the wrong type.

typeshed contract: action is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import Option
obj = object.__new__(Option)
try:
    obj.take_action(12345, "", "", None, None, None)  # action: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/TitledHelpFormatter__format_heading__heading_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_TitledHelpFormatter__format_heading__heading_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "TitledHelpFormatter__format_heading__heading_as_str_wrong"
# subject = "optparse.TitledHelpFormatter.format_heading(heading: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.TitledHelpFormatter.format_heading(heading: str); call it with the wrong type.

typeshed contract: heading is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import TitledHelpFormatter
obj = object.__new__(TitledHelpFormatter)
try:
    obj.format_heading(12345)  # heading: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/TitledHelpFormatter__format_usage__usage_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_TitledHelpFormatter__format_usage__usage_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "TitledHelpFormatter__format_usage__usage_as_str_wrong"
# subject = "optparse.TitledHelpFormatter.format_usage(usage: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.TitledHelpFormatter.format_usage(usage: str); call it with the wrong type.

typeshed contract: usage is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import TitledHelpFormatter
obj = object.__new__(TitledHelpFormatter)
try:
    obj.format_usage(12345)  # usage: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/TitledHelpFormatter__init__indent_increment_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_TitledHelpFormatter__init__indent_increment_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "TitledHelpFormatter__init__indent_increment_as_int_wrong"
# subject = "optparse.TitledHelpFormatter.__init__(indent_increment: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.TitledHelpFormatter.__init__(indent_increment: int); call it with the wrong type.

typeshed contract: indent_increment is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import TitledHelpFormatter
try:
    TitledHelpFormatter("not_an_int")  # indent_increment: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/Values____getattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_Values____getattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "Values____getattr____name_as_str_wrong"
# subject = "optparse.Values.__getattr__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.Values.__getattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import Values
obj = object.__new__(Values)
try:
    obj.__getattr__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/Values____setattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_Values____setattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "Values____setattr____name_as_str_wrong"
# subject = "optparse.Values.__setattr__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.Values.__setattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import Values
obj = object.__new__(Values)
try:
    obj.__setattr__(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/Values__ensure_value__attr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_Values__ensure_value__attr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "Values__ensure_value__attr_as_str_wrong"
# subject = "optparse.Values.ensure_value(attr: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.Values.ensure_value(attr: str); call it with the wrong type.

typeshed contract: attr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import Values
obj = object.__new__(Values)
try:
    obj.ensure_value(12345, None)  # attr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/Values__init__defaults_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_Values__init__defaults_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "Values__init__defaults_as_typed_wrong"
# subject = "optparse.Values.__init__(defaults: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.Values.__init__(defaults: typed); call it with the wrong type.

typeshed contract: defaults is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import Values
try:
    Values(_W())  # defaults: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/Values__read_file__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_Values__read_file__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "Values__read_file__filename_as_str_wrong"
# subject = "optparse.Values.read_file(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.Values.read_file(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import Values
obj = object.__new__(Values)
try:
    obj.read_file(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/Values__read_module__modname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_Values__read_module__modname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "Values__read_module__modname_as_str_wrong"
# subject = "optparse.Values.read_module(modname: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.Values.read_module(modname: str); call it with the wrong type.

typeshed contract: modname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import Values
obj = object.__new__(Values)
try:
    obj.read_module(12345)  # modname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/check_builtin__option_as_Option_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_check_builtin__option_as_Option_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "check_builtin__option_as_Option_wrong"
# subject = "optparse.check_builtin(option: Option)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.check_builtin(option: Option); call it with the wrong type.

typeshed contract: option is Option. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import check_builtin
try:
    check_builtin(_W(), "", "")  # option: Option <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/optparse/check_choice__option_as_Option_wrong.py`.
#[test]
fn test_gen_type_std_libs_optparse_check_choice__option_as_Option_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "check_choice__option_as_Option_wrong"
# subject = "optparse.check_choice(option: Option)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.check_choice(option: Option); call it with the wrong type.

typeshed contract: option is Option. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import check_choice
try:
    check_choice(_W(), "", "")  # option: Option <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
