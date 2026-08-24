use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/urllib_parse/ipv6_missing_close_bracket_raises.py`.
#[test]
fn test_gen_errors_std_libs_urllib_parse_ipv6_missing_close_bracket_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "ipv6_missing_close_bracket_raises"
# subject = "urllib.parse.urlsplit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: ipv6_missing_close_bracket_raises (errors)."""
from urllib.parse import urlsplit

_raised = False
try:
    urlsplit('scheme://[v6a.ip')
except ValueError:
    _raised = True
assert _raised, "ipv6_missing_close_bracket_raises: expected ValueError"
print("ipv6_missing_close_bracket_raises OK")
"###);
    assert_output(&out, r###"ipv6_missing_close_bracket_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/urllib_parse/ipv6_missing_open_bracket_raises.py`.
#[test]
fn test_gen_errors_std_libs_urllib_parse_ipv6_missing_open_bracket_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "ipv6_missing_open_bracket_raises"
# subject = "urllib.parse.urlsplit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: ipv6_missing_open_bracket_raises (errors)."""
from urllib.parse import urlsplit

_raised = False
try:
    urlsplit('scheme://v6a.ip]')
except ValueError:
    _raised = True
assert _raised, "ipv6_missing_open_bracket_raises: expected ValueError"
print("ipv6_missing_open_bracket_raises OK")
"###);
    assert_output(&out, r###"ipv6_missing_open_bracket_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/urllib_parse/parse_qsl_empty_separator_raises.py`.
#[test]
fn test_gen_errors_std_libs_urllib_parse_parse_qsl_empty_separator_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "parse_qsl_empty_separator_raises"
# subject = "urllib.parse.parse_qsl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.parse_qsl: parse_qsl_empty_separator_raises (errors)."""
from urllib.parse import parse_qsl

_raised = False
try:
    parse_qsl('a=b', separator='')
except ValueError:
    _raised = True
assert _raised, "parse_qsl_empty_separator_raises: expected ValueError"
print("parse_qsl_empty_separator_raises OK")
"###);
    assert_output(&out, r###"parse_qsl_empty_separator_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/urllib_parse/parse_qsl_max_num_fields_raises.py`.
#[test]
fn test_gen_errors_std_libs_urllib_parse_parse_qsl_max_num_fields_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "parse_qsl_max_num_fields_raises"
# subject = "urllib.parse.parse_qsl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.parse_qsl: parse_qsl_max_num_fields_raises (errors)."""
from urllib.parse import parse_qsl

_raised = False
try:
    parse_qsl('&'.join(['a=a'] * 11), max_num_fields=10)
except ValueError:
    _raised = True
assert _raised, "parse_qsl_max_num_fields_raises: expected ValueError"
print("parse_qsl_max_num_fields_raises OK")
"###);
    assert_output(&out, r###"parse_qsl_max_num_fields_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/urllib_parse/parse_qsl_strict_bad_pair_raises.py`.
#[test]
fn test_gen_errors_std_libs_urllib_parse_parse_qsl_strict_bad_pair_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "parse_qsl_strict_bad_pair_raises"
# subject = "urllib.parse.parse_qsl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.parse_qsl: parse_qsl_strict_bad_pair_raises (errors)."""
from urllib.parse import parse_qsl

_raised = False
try:
    parse_qsl('novalkey', strict_parsing=True)
except ValueError:
    _raised = True
assert _raised, "parse_qsl_strict_bad_pair_raises: expected ValueError"
print("parse_qsl_strict_bad_pair_raises OK")
"###);
    assert_output(&out, r###"parse_qsl_strict_bad_pair_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/urllib_parse/port_non_numeric_raises.py`.
#[test]
fn test_gen_errors_std_libs_urllib_parse_port_non_numeric_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "port_non_numeric_raises"
# subject = "urllib.parse.urlparse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: port_non_numeric_raises (errors)."""
from urllib.parse import urlparse

_raised = False
try:
    urlparse('http://Server=sde; Service=sde:oracle').port
except ValueError:
    _raised = True
assert _raised, "port_non_numeric_raises: expected ValueError"
print("port_non_numeric_raises OK")
"###);
    assert_output(&out, r###"port_non_numeric_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/urllib_parse/port_out_of_range_raises.py`.
#[test]
fn test_gen_errors_std_libs_urllib_parse_port_out_of_range_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "port_out_of_range_raises"
# subject = "urllib.parse.urlsplit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: port_out_of_range_raises (errors)."""
from urllib.parse import urlsplit

_raised = False
try:
    urlsplit('http://host:65536/doc/').port
except ValueError:
    _raised = True
assert _raised, "port_out_of_range_raises: expected ValueError"
print("port_out_of_range_raises OK")
"###);
    assert_output(&out, r###"port_out_of_range_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/urllib_parse/urlunsplit_mix_str_bytes_raises.py`.
#[test]
fn test_gen_errors_std_libs_urllib_parse_urlunsplit_mix_str_bytes_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "urlunsplit_mix_str_bytes_raises"
# subject = "urllib.parse.urlunsplit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlunsplit: urlunsplit_mix_str_bytes_raises (errors)."""
from urllib.parse import urlunsplit

_raised = False
try:
    urlunsplit(('http', b'h', '', '', ''))
except TypeError:
    _raised = True
assert _raised, "urlunsplit_mix_str_bytes_raises: expected TypeError"
print("urlunsplit_mix_str_bytes_raises OK")
"###);
    assert_output(&out, r###"urlunsplit_mix_str_bytes_raises OK
"###);
}
