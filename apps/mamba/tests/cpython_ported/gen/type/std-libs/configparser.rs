use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/configparser/ConfigParser__get__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_ConfigParser__get__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "ConfigParser__get__section_as__SectionName_wrong"
# subject = "configparser.ConfigParser.get(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.ConfigParser.get(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import ConfigParser
obj = object.__new__(ConfigParser)
try:
    obj.get(_W(), "")  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/ConverterMapping____delitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_ConverterMapping____delitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "ConverterMapping____delitem____key_as_str_wrong"
# subject = "configparser.ConverterMapping.__delitem__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.ConverterMapping.__delitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import ConverterMapping
obj = object.__new__(ConverterMapping)
try:
    obj.__delitem__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/ConverterMapping____getitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_ConverterMapping____getitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "ConverterMapping____getitem____key_as_str_wrong"
# subject = "configparser.ConverterMapping.__getitem__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.ConverterMapping.__getitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import ConverterMapping
obj = object.__new__(ConverterMapping)
try:
    obj.__getitem__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/ConverterMapping____setitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_ConverterMapping____setitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "ConverterMapping____setitem____key_as_str_wrong"
# subject = "configparser.ConverterMapping.__setitem__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.ConverterMapping.__setitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import ConverterMapping
obj = object.__new__(ConverterMapping)
try:
    obj.__setitem__(12345, None)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/ConverterMapping__init__parser_as_RawConfigParser_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_ConverterMapping__init__parser_as_RawConfigParser_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "ConverterMapping__init__parser_as_RawConfigParser_wrong"
# subject = "configparser.ConverterMapping.__init__(parser: RawConfigParser)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.ConverterMapping.__init__(parser: RawConfigParser); call it with the wrong type.

typeshed contract: parser is RawConfigParser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import ConverterMapping
try:
    ConverterMapping(_W())  # parser: RawConfigParser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/DuplicateOptionError__init__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_DuplicateOptionError__init__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "DuplicateOptionError__init__section_as__SectionName_wrong"
# subject = "configparser.DuplicateOptionError.__init__(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.DuplicateOptionError.__init__(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import DuplicateOptionError
try:
    DuplicateOptionError(_W(), "")  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/DuplicateSectionError__init__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_DuplicateSectionError__init__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "DuplicateSectionError__init__section_as__SectionName_wrong"
# subject = "configparser.DuplicateSectionError.__init__(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.DuplicateSectionError.__init__(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import DuplicateSectionError
try:
    DuplicateSectionError(_W())  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/Error__init__msg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_Error__init__msg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "Error__init__msg_as_str_wrong"
# subject = "configparser.Error.__init__(msg: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.Error.__init__(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import Error
try:
    Error(12345)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/InterpolationDepthError__init__option_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_InterpolationDepthError__init__option_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "InterpolationDepthError__init__option_as_str_wrong"
# subject = "configparser.InterpolationDepthError.__init__(option: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.InterpolationDepthError.__init__(option: str); call it with the wrong type.

typeshed contract: option is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import InterpolationDepthError
try:
    InterpolationDepthError(12345, None, None)  # option: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/InterpolationError__init__option_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_InterpolationError__init__option_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "InterpolationError__init__option_as_str_wrong"
# subject = "configparser.InterpolationError.__init__(option: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.InterpolationError.__init__(option: str); call it with the wrong type.

typeshed contract: option is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import InterpolationError
try:
    InterpolationError(12345, None, "")  # option: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/InterpolationMissingOptionError__init__option_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_InterpolationMissingOptionError__init__option_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "InterpolationMissingOptionError__init__option_as_str_wrong"
# subject = "configparser.InterpolationMissingOptionError.__init__(option: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.InterpolationMissingOptionError.__init__(option: str); call it with the wrong type.

typeshed contract: option is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import InterpolationMissingOptionError
try:
    InterpolationMissingOptionError(12345, None, None, "")  # option: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/Interpolation__before_get__parser_as__Parser_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_Interpolation__before_get__parser_as__Parser_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "Interpolation__before_get__parser_as__Parser_wrong"
# subject = "configparser.Interpolation.before_get(parser: _Parser)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.Interpolation.before_get(parser: _Parser); call it with the wrong type.

typeshed contract: parser is _Parser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import Interpolation
obj = object.__new__(Interpolation)
try:
    obj.before_get(_W(), None, "", "", None)  # parser: _Parser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/Interpolation__before_read__parser_as__Parser_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_Interpolation__before_read__parser_as__Parser_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "Interpolation__before_read__parser_as__Parser_wrong"
# subject = "configparser.Interpolation.before_read(parser: _Parser)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.Interpolation.before_read(parser: _Parser); call it with the wrong type.

typeshed contract: parser is _Parser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import Interpolation
obj = object.__new__(Interpolation)
try:
    obj.before_read(_W(), None, "", "")  # parser: _Parser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/Interpolation__before_set__parser_as__Parser_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_Interpolation__before_set__parser_as__Parser_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "Interpolation__before_set__parser_as__Parser_wrong"
# subject = "configparser.Interpolation.before_set(parser: _Parser)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.Interpolation.before_set(parser: _Parser); call it with the wrong type.

typeshed contract: parser is _Parser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import Interpolation
obj = object.__new__(Interpolation)
try:
    obj.before_set(_W(), None, "", "")  # parser: _Parser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/Interpolation__before_write__parser_as__Parser_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_Interpolation__before_write__parser_as__Parser_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "Interpolation__before_write__parser_as__Parser_wrong"
# subject = "configparser.Interpolation.before_write(parser: _Parser)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.Interpolation.before_write(parser: _Parser); call it with the wrong type.

typeshed contract: parser is _Parser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import Interpolation
obj = object.__new__(Interpolation)
try:
    obj.before_write(_W(), None, "", "")  # parser: _Parser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/LegacyInterpolation__before_get__parser_as__Parser_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_LegacyInterpolation__before_get__parser_as__Parser_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "LegacyInterpolation__before_get__parser_as__Parser_wrong"
# subject = "configparser.LegacyInterpolation.before_get(parser: _Parser)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.LegacyInterpolation.before_get(parser: _Parser); call it with the wrong type.

typeshed contract: parser is _Parser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import LegacyInterpolation
obj = object.__new__(LegacyInterpolation)
try:
    obj.before_get(_W(), None, "", "", None)  # parser: _Parser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/MissingSectionHeaderError__init__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_MissingSectionHeaderError__init__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "MissingSectionHeaderError__init__filename_as_str_wrong"
# subject = "configparser.MissingSectionHeaderError.__init__(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.MissingSectionHeaderError.__init__(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import MissingSectionHeaderError
try:
    MissingSectionHeaderError(12345, 0, "")  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/MultilineContinuationError__init__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_MultilineContinuationError__init__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "MultilineContinuationError__init__filename_as_str_wrong"
# subject = "configparser.MultilineContinuationError.__init__(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.MultilineContinuationError.__init__(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import MultilineContinuationError
try:
    MultilineContinuationError(12345, 0, "")  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/NoOptionError__init__option_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_NoOptionError__init__option_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "NoOptionError__init__option_as_str_wrong"
# subject = "configparser.NoOptionError.__init__(option: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.NoOptionError.__init__(option: str); call it with the wrong type.

typeshed contract: option is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import NoOptionError
try:
    NoOptionError(12345, None)  # option: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/NoSectionError__init__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_NoSectionError__init__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "NoSectionError__init__section_as__SectionName_wrong"
# subject = "configparser.NoSectionError.__init__(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.NoSectionError.__init__(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import NoSectionError
try:
    NoSectionError(_W())  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/ParsingError__append__lineno_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_ParsingError__append__lineno_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "ParsingError__append__lineno_as_int_wrong"
# subject = "configparser.ParsingError.append(lineno: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.ParsingError.append(lineno: int); call it with the wrong type.

typeshed contract: lineno is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import ParsingError
obj = object.__new__(ParsingError)
try:
    obj.append("not_an_int", "")  # lineno: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/ParsingError__combine__others_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_ParsingError__combine__others_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "ParsingError__combine__others_as_Iterable_wrong"
# subject = "configparser.ParsingError.combine(others: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.ParsingError.combine(others: Iterable); call it with the wrong type.

typeshed contract: others is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import ParsingError
obj = object.__new__(ParsingError)
try:
    obj.combine(_W())  # others: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/ParsingError__init__source_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_ParsingError__init__source_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "ParsingError__init__source_as_str_wrong"
# subject = "configparser.ParsingError.__init__(source: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.ParsingError.__init__(source: str); call it with the wrong type.

typeshed contract: source is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import ParsingError
try:
    ParsingError(12345)  # source: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/ParsingError__init__source_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_ParsingError__init__source_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "ParsingError__init__source_as_typed_wrong"
# subject = "configparser.ParsingError.__init__(source: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.ParsingError.__init__(source: typed); call it with the wrong type.

typeshed contract: source is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import ParsingError
try:
    ParsingError(_W(), None)  # source: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser____delitem____key_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser____delitem____key_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser____delitem____key_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.__delitem__(key: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.__delitem__(key: _SectionName); call it with the wrong type.

typeshed contract: key is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.__delitem__(_W())  # key: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser____getitem____key_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser____getitem____key_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser____getitem____key_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.__getitem__(key: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.__getitem__(key: _SectionName); call it with the wrong type.

typeshed contract: key is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.__getitem__(_W())  # key: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser____setitem____key_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser____setitem____key_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser____setitem____key_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.__setitem__(key: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.__setitem__(key: _SectionName); call it with the wrong type.

typeshed contract: key is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.__setitem__(_W(), None)  # key: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__add_section__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__add_section__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__add_section__section_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.add_section(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.add_section(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.add_section(_W())  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__get__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__get__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__get__section_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.get(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.get(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = RawConfigParser()
try:
    obj.get(_W(), "")  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__getboolean__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__getboolean__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__getboolean__section_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.getboolean(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.getboolean(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = RawConfigParser()
try:
    obj.getboolean(_W(), "")  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__getfloat__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__getfloat__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__getfloat__section_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.getfloat(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.getfloat(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = RawConfigParser()
try:
    obj.getfloat(_W(), "")  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__getint__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__getint__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__getint__section_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.getint(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.getint(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = RawConfigParser()
try:
    obj.getint(_W(), "")  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__has_option__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__has_option__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__has_option__section_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.has_option(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.has_option(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.has_option(_W(), "")  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__has_section__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__has_section__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__has_section__section_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.has_section(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.has_section(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.has_section(_W())  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__init__defaults_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__init__defaults_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__init__defaults_as_typed_wrong"
# subject = "configparser.RawConfigParser.__init__(defaults: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.__init__(defaults: typed); call it with the wrong type.

typeshed contract: defaults is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
try:
    RawConfigParser(_W())  # defaults: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__items__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__items__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__items__section_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.items(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.items(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = RawConfigParser()
try:
    obj.items(_W())  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__options__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__options__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__options__section_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.options(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.options(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.options(_W())  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__optionxform__optionstr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__optionxform__optionstr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__optionxform__optionstr_as_str_wrong"
# subject = "configparser.RawConfigParser.optionxform(optionstr: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.optionxform(optionstr: str); call it with the wrong type.

typeshed contract: optionstr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.optionxform(12345)  # optionstr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__read__filenames_as_GenericPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__read__filenames_as_GenericPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__read__filenames_as_GenericPath_wrong"
# subject = "configparser.RawConfigParser.read(filenames: GenericPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.read(filenames: GenericPath); call it with the wrong type.

typeshed contract: filenames is GenericPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = RawConfigParser()
try:
    obj.read(_W())  # filenames: GenericPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__read__filenames_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__read__filenames_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__read__filenames_as_Iterable_wrong"
# subject = "configparser.RawConfigParser.read(filenames: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.read(filenames: Iterable); call it with the wrong type.

typeshed contract: filenames is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = RawConfigParser()
try:
    obj.read(_W())  # filenames: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__read_dict__dictionary_as_Mapping_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__read_dict__dictionary_as_Mapping_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__read_dict__dictionary_as_Mapping_wrong"
# subject = "configparser.RawConfigParser.read_dict(dictionary: Mapping)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.read_dict(dictionary: Mapping); call it with the wrong type.

typeshed contract: dictionary is Mapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = RawConfigParser()
try:
    obj.read_dict(_W())  # dictionary: Mapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__read_file__f_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__read_file__f_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__read_file__f_as_Iterable_wrong"
# subject = "configparser.RawConfigParser.read_file(f: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.read_file(f: Iterable); call it with the wrong type.

typeshed contract: f is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.read_file(_W())  # f: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__read_string__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__read_string__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__read_string__string_as_str_wrong"
# subject = "configparser.RawConfigParser.read_string(string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.read_string(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.read_string(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__readfp__fp_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__readfp__fp_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__readfp__fp_as_Iterable_wrong"
# subject = "configparser.RawConfigParser.readfp(fp: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.readfp(fp: Iterable); call it with the wrong type.

typeshed contract: fp is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.readfp(_W())  # fp: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__remove_option__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__remove_option__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__remove_option__section_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.remove_option(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.remove_option(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.remove_option(_W(), "")  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__remove_section__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__remove_section__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__remove_section__section_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.remove_section(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.remove_section(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.remove_section(_W())  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__set__section_as__SectionName_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__set__section_as__SectionName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__set__section_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.set(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.set(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.set(_W(), "")  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/RawConfigParser__write__fp_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_RawConfigParser__write__fp_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__write__fp_as_SupportsWrite_wrong"
# subject = "configparser.RawConfigParser.write(fp: SupportsWrite)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.write(fp: SupportsWrite); call it with the wrong type.

typeshed contract: fp is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.write(_W())  # fp: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/SectionProxy____delitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_SectionProxy____delitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "SectionProxy____delitem____key_as_str_wrong"
# subject = "configparser.SectionProxy.__delitem__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.SectionProxy.__delitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import SectionProxy
obj = object.__new__(SectionProxy)
try:
    obj.__delitem__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/SectionProxy____getattr____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_SectionProxy____getattr____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "SectionProxy____getattr____key_as_str_wrong"
# subject = "configparser.SectionProxy.__getattr__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.SectionProxy.__getattr__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import SectionProxy
obj = object.__new__(SectionProxy)
try:
    obj.__getattr__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/SectionProxy____getitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_SectionProxy____getitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "SectionProxy____getitem____key_as_str_wrong"
# subject = "configparser.SectionProxy.__getitem__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.SectionProxy.__getitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import SectionProxy
obj = object.__new__(SectionProxy)
try:
    obj.__getitem__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/SectionProxy____setitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_SectionProxy____setitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "SectionProxy____setitem____key_as_str_wrong"
# subject = "configparser.SectionProxy.__setitem__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.SectionProxy.__setitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import SectionProxy
obj = object.__new__(SectionProxy)
try:
    obj.__setitem__(12345, "")  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/SectionProxy__get__option_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_SectionProxy__get__option_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "SectionProxy__get__option_as_str_wrong"
# subject = "configparser.SectionProxy.get(option: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.SectionProxy.get(option: str); call it with the wrong type.

typeshed contract: option is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import SectionProxy
obj = object.__new__(SectionProxy)
try:
    obj.get(12345)  # option: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/SectionProxy__getboolean__option_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_SectionProxy__getboolean__option_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "SectionProxy__getboolean__option_as_str_wrong"
# subject = "configparser.SectionProxy.getboolean(option: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.SectionProxy.getboolean(option: str); call it with the wrong type.

typeshed contract: option is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import SectionProxy
obj = object.__new__(SectionProxy)
try:
    obj.getboolean(12345)  # option: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/SectionProxy__getfloat__option_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_SectionProxy__getfloat__option_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "SectionProxy__getfloat__option_as_str_wrong"
# subject = "configparser.SectionProxy.getfloat(option: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.SectionProxy.getfloat(option: str); call it with the wrong type.

typeshed contract: option is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import SectionProxy
obj = object.__new__(SectionProxy)
try:
    obj.getfloat(12345)  # option: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/SectionProxy__getint__option_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_SectionProxy__getint__option_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "SectionProxy__getint__option_as_str_wrong"
# subject = "configparser.SectionProxy.getint(option: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.SectionProxy.getint(option: str); call it with the wrong type.

typeshed contract: option is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import SectionProxy
obj = object.__new__(SectionProxy)
try:
    obj.getint(12345)  # option: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/configparser/SectionProxy__init__parser_as_RawConfigParser_wrong.py`.
#[test]
fn test_gen_type_std_libs_configparser_SectionProxy__init__parser_as_RawConfigParser_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "SectionProxy__init__parser_as_RawConfigParser_wrong"
# subject = "configparser.SectionProxy.__init__(parser: RawConfigParser)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.SectionProxy.__init__(parser: RawConfigParser); call it with the wrong type.

typeshed contract: parser is RawConfigParser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import SectionProxy
try:
    SectionProxy(_W(), "")  # parser: RawConfigParser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
