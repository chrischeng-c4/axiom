use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/configparser/add_duplicate_section_raises.py`.
#[test]
fn test_gen_errors_std_libs_configparser_add_duplicate_section_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "add_duplicate_section_raises"
# subject = "configparser.ConfigParser.add_section"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.add_section: add_duplicate_section_raises (errors)."""
import configparser
_cp_dup_sec = configparser.ConfigParser()
_cp_dup_sec.read_string('[s1]\nk=v\n')

_raised = False
try:
    _cp_dup_sec.add_section('s1')
except configparser.DuplicateSectionError:
    _raised = True
assert _raised, "add_duplicate_section_raises: expected configparser.DuplicateSectionError"
print("add_duplicate_section_raises OK")
"###);
    assert_output(&out, r###"add_duplicate_section_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/configparser/duplicate_option_in_section_raises.py`.
#[test]
fn test_gen_errors_std_libs_configparser_duplicate_option_in_section_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "duplicate_option_in_section_raises"
# subject = "configparser.ConfigParser.read_string"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.read_string: duplicate_option_in_section_raises (errors)."""
import configparser

_raised = False
try:
    configparser.ConfigParser().read_string('[s]\na = 1\na = 2\n')
except configparser.DuplicateOptionError:
    _raised = True
assert _raised, "duplicate_option_in_section_raises: expected configparser.DuplicateOptionError"
print("duplicate_option_in_section_raises OK")
"###);
    assert_output(&out, r###"duplicate_option_in_section_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/configparser/get_missing_option_raises.py`.
#[test]
fn test_gen_errors_std_libs_configparser_get_missing_option_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "get_missing_option_raises"
# subject = "configparser.ConfigParser.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: get_missing_option_raises (errors)."""
import configparser
_cp_no_opt = configparser.ConfigParser()
_cp_no_opt.read_string('[s1]\nkey=val\n')

_raised = False
try:
    _cp_no_opt.get('s1', 'nokey')
except configparser.NoOptionError:
    _raised = True
assert _raised, "get_missing_option_raises: expected configparser.NoOptionError"
print("get_missing_option_raises OK")
"###);
    assert_output(&out, r###"get_missing_option_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/configparser/get_missing_section_raises.py`.
#[test]
fn test_gen_errors_std_libs_configparser_get_missing_section_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "get_missing_section_raises"
# subject = "configparser.ConfigParser.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: get_missing_section_raises (errors)."""
import configparser

_raised = False
try:
    configparser.ConfigParser().get('nosection', 'key')
except configparser.NoSectionError:
    _raised = True
assert _raised, "get_missing_section_raises: expected configparser.NoSectionError"
print("get_missing_section_raises OK")
"###);
    assert_output(&out, r###"get_missing_section_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/configparser/interpolation_cycle_exceeds_depth_raises.py`.
#[test]
fn test_gen_errors_std_libs_configparser_interpolation_cycle_exceeds_depth_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "interpolation_cycle_exceeds_depth_raises"
# subject = "configparser.ConfigParser.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: interpolation_cycle_exceeds_depth_raises (errors)."""
import configparser
_cp_cyc = configparser.ConfigParser()
_cp_cyc.read_string('[s]\na = %(b)s\nb = %(a)s\n')

_raised = False
try:
    _cp_cyc.get('s', 'a')
except configparser.InterpolationDepthError:
    _raised = True
assert _raised, "interpolation_cycle_exceeds_depth_raises: expected configparser.InterpolationDepthError"
print("interpolation_cycle_exceeds_depth_raises OK")
"###);
    assert_output(&out, r###"interpolation_cycle_exceeds_depth_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/configparser/interpolation_missing_referent_raises.py`.
#[test]
fn test_gen_errors_std_libs_configparser_interpolation_missing_referent_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "interpolation_missing_referent_raises"
# subject = "configparser.ConfigParser.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: interpolation_missing_referent_raises (errors)."""
import configparser
_cp_miss = configparser.ConfigParser()
_cp_miss.read_string('[s]\nv = %(absent)s\n')

_raised = False
try:
    _cp_miss.get('s', 'v')
except configparser.InterpolationMissingOptionError:
    _raised = True
assert _raised, "interpolation_missing_referent_raises: expected configparser.InterpolationMissingOptionError"
print("interpolation_missing_referent_raises OK")
"###);
    assert_output(&out, r###"interpolation_missing_referent_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/configparser/keys_before_section_header_raises.py`.
#[test]
fn test_gen_errors_std_libs_configparser_keys_before_section_header_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "keys_before_section_header_raises"
# subject = "configparser.ConfigParser.read_string"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.read_string: keys_before_section_header_raises (errors)."""
import configparser

_raised = False
try:
    configparser.ConfigParser().read_string('key = value\n')
except configparser.MissingSectionHeaderError:
    _raised = True
assert _raised, "keys_before_section_header_raises: expected configparser.MissingSectionHeaderError"
print("keys_before_section_header_raises OK")
"###);
    assert_output(&out, r###"keys_before_section_header_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/configparser/set_none_without_allow_no_value_raises.py`.
#[test]
fn test_gen_errors_std_libs_configparser_set_none_without_allow_no_value_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "set_none_without_allow_no_value_raises"
# subject = "configparser.ConfigParser.set"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.set: set_none_without_allow_no_value_raises (errors)."""
import configparser
_cp_set_none = configparser.ConfigParser(allow_no_value=False)
_cp_set_none.add_section('s')

_raised = False
try:
    _cp_set_none.set('s', 'opt', None)
except TypeError:
    _raised = True
assert _raised, "set_none_without_allow_no_value_raises: expected TypeError"
print("set_none_without_allow_no_value_raises OK")
"###);
    assert_output(&out, r###"set_none_without_allow_no_value_raises OK
"###);
}
