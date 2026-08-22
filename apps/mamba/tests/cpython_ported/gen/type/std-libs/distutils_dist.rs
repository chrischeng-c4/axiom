use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/distutils_dist/DistributionMetadata__init__path_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dist_DistributionMetadata__init__path_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "DistributionMetadata__init__path_as_typed_wrong"
# subject = "distutils.dist.DistributionMetadata.__init__(path: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.DistributionMetadata.__init__(path: typed); call it with the wrong type.

typeshed contract: path is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dist import DistributionMetadata
try:
    DistributionMetadata(_W())  # path: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dist/DistributionMetadata__read_pkg_file__file_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dist_DistributionMetadata__read_pkg_file__file_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "DistributionMetadata__read_pkg_file__file_as_IO_wrong"
# subject = "distutils.dist.DistributionMetadata.read_pkg_file(file: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.DistributionMetadata.read_pkg_file(file: IO); call it with the wrong type.

typeshed contract: file is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dist import DistributionMetadata
obj = object.__new__(DistributionMetadata)
try:
    obj.read_pkg_file(_W())  # file: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dist/DistributionMetadata__set_obsoletes__value_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dist_DistributionMetadata__set_obsoletes__value_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "DistributionMetadata__set_obsoletes__value_as_Iterable_wrong"
# subject = "distutils.dist.DistributionMetadata.set_obsoletes(value: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.DistributionMetadata.set_obsoletes(value: Iterable); call it with the wrong type.

typeshed contract: value is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dist import DistributionMetadata
obj = object.__new__(DistributionMetadata)
try:
    obj.set_obsoletes(_W())  # value: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dist/DistributionMetadata__set_provides__value_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dist_DistributionMetadata__set_provides__value_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "DistributionMetadata__set_provides__value_as_Iterable_wrong"
# subject = "distutils.dist.DistributionMetadata.set_provides(value: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.DistributionMetadata.set_provides(value: Iterable); call it with the wrong type.

typeshed contract: value is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dist import DistributionMetadata
obj = object.__new__(DistributionMetadata)
try:
    obj.set_provides(_W())  # value: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dist/DistributionMetadata__set_requires__value_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dist_DistributionMetadata__set_requires__value_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "DistributionMetadata__set_requires__value_as_Iterable_wrong"
# subject = "distutils.dist.DistributionMetadata.set_requires(value: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.DistributionMetadata.set_requires(value: Iterable); call it with the wrong type.

typeshed contract: value is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dist import DistributionMetadata
obj = object.__new__(DistributionMetadata)
try:
    obj.set_requires(_W())  # value: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dist/DistributionMetadata__write_pkg_file__file_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dist_DistributionMetadata__write_pkg_file__file_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "DistributionMetadata__write_pkg_file__file_as_SupportsWrite_wrong"
# subject = "distutils.dist.DistributionMetadata.write_pkg_file(file: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.DistributionMetadata.write_pkg_file(file: SupportsWrite); call it with the wrong type.

typeshed contract: file is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dist import DistributionMetadata
obj = object.__new__(DistributionMetadata)
try:
    obj.write_pkg_file(_W())  # file: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dist/DistributionMetadata__write_pkg_info__base_dir_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dist_DistributionMetadata__write_pkg_info__base_dir_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "DistributionMetadata__write_pkg_info__base_dir_as_StrPath_wrong"
# subject = "distutils.dist.DistributionMetadata.write_pkg_info(base_dir: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.DistributionMetadata.write_pkg_info(base_dir: StrPath); call it with the wrong type.

typeshed contract: base_dir is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dist import DistributionMetadata
obj = object.__new__(DistributionMetadata)
try:
    obj.write_pkg_info(_W())  # base_dir: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dist/Distribution__announce__level_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dist_Distribution__announce__level_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "Distribution__announce__level_as_int_wrong"
# subject = "distutils.dist.Distribution.announce(level: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.Distribution.announce(level: int); call it with the wrong type.

typeshed contract: level is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.dist import Distribution
obj = object.__new__(Distribution)
try:
    obj.announce(None, "not_an_int")  # level: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dist/Distribution__dump_option_dicts__indent_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dist_Distribution__dump_option_dicts__indent_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "Distribution__dump_option_dicts__indent_as_str_wrong"
# subject = "distutils.dist.Distribution.dump_option_dicts(indent: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.Distribution.dump_option_dicts(indent: str); call it with the wrong type.

typeshed contract: indent is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.dist import Distribution
obj = object.__new__(Distribution)
try:
    obj.dump_option_dicts(None, None, 12345)  # indent: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dist/Distribution__get_option_dict__command_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dist_Distribution__get_option_dict__command_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "Distribution__get_option_dict__command_as_str_wrong"
# subject = "distutils.dist.Distribution.get_option_dict(command: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.Distribution.get_option_dict(command: str); call it with the wrong type.

typeshed contract: command is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.dist import Distribution
obj = object.__new__(Distribution)
try:
    obj.get_option_dict(12345)  # command: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dist/Distribution__init__attrs_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dist_Distribution__init__attrs_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "Distribution__init__attrs_as_typed_wrong"
# subject = "distutils.dist.Distribution.__init__(attrs: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.Distribution.__init__(attrs: typed); call it with the wrong type.

typeshed contract: attrs is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dist import Distribution
try:
    Distribution(_W())  # attrs: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dist/Distribution__parse_config_files__filenames_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dist_Distribution__parse_config_files__filenames_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "Distribution__parse_config_files__filenames_as_typed_wrong"
# subject = "distutils.dist.Distribution.parse_config_files(filenames: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.Distribution.parse_config_files(filenames: typed); call it with the wrong type.

typeshed contract: filenames is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dist import Distribution
obj = object.__new__(Distribution)
try:
    obj.parse_config_files(_W())  # filenames: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dist/Distribution__run_command__command_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dist_Distribution__run_command__command_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "Distribution__run_command__command_as_str_wrong"
# subject = "distutils.dist.Distribution.run_command(command: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.Distribution.run_command(command: str); call it with the wrong type.

typeshed contract: command is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.dist import Distribution
obj = object.__new__(Distribution)
try:
    obj.run_command(12345)  # command: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
