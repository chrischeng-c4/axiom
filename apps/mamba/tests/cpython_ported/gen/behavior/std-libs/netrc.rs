use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_comment_after_machine_line.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_comment_after_machine_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_comment_after_machine_line"
# subject = "cpython.test_netrc.NetrcTestCase.test_comment_after_machine_line"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_comment_after_machine_line
"""Auto-ported test: NetrcTestCase::test_comment_after_machine_line (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_comment('            machine foo.domain.com login bar password pass\n            # comment\n            machine bar.domain.com login foo password pass\n            ')
_test_comment('            machine foo.domain.com login bar password pass\n            machine bar.domain.com login foo password pass\n            # comment\n            ')
print("NetrcTestCase::test_comment_after_machine_line: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_comment_after_machine_line: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_comment_after_machine_line_hash_only.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_comment_after_machine_line_hash_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_comment_after_machine_line_hash_only"
# subject = "cpython.test_netrc.NetrcTestCase.test_comment_after_machine_line_hash_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_comment_after_machine_line_hash_only
"""Auto-ported test: NetrcTestCase::test_comment_after_machine_line_hash_only (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_comment('            machine foo.domain.com login bar password pass\n            #\n            machine bar.domain.com login foo password pass\n            ')
_test_comment('            machine foo.domain.com login bar password pass\n            machine bar.domain.com login foo password pass\n            #\n            ')
print("NetrcTestCase::test_comment_after_machine_line_hash_only: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_comment_after_machine_line_hash_only: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_comment_after_machine_line_no_space.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_comment_after_machine_line_no_space() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_comment_after_machine_line_no_space"
# subject = "cpython.test_netrc.NetrcTestCase.test_comment_after_machine_line_no_space"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_comment_after_machine_line_no_space
"""Auto-ported test: NetrcTestCase::test_comment_after_machine_line_no_space (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_comment('            machine foo.domain.com login bar password pass\n            #comment\n            machine bar.domain.com login foo password pass\n            ')
_test_comment('            machine foo.domain.com login bar password pass\n            machine bar.domain.com login foo password pass\n            #comment\n            ')
print("NetrcTestCase::test_comment_after_machine_line_no_space: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_comment_after_machine_line_no_space: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_comment_at_end_of_machine_line.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_comment_at_end_of_machine_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_comment_at_end_of_machine_line"
# subject = "cpython.test_netrc.NetrcTestCase.test_comment_at_end_of_machine_line"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_comment_at_end_of_machine_line
"""Auto-ported test: NetrcTestCase::test_comment_at_end_of_machine_line (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_comment('            machine foo.domain.com login bar password pass # comment\n            machine bar.domain.com login foo password pass\n            ')
print("NetrcTestCase::test_comment_at_end_of_machine_line: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_comment_at_end_of_machine_line: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_comment_at_end_of_machine_line_no_space.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_comment_at_end_of_machine_line_no_space() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_comment_at_end_of_machine_line_no_space"
# subject = "cpython.test_netrc.NetrcTestCase.test_comment_at_end_of_machine_line_no_space"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_comment_at_end_of_machine_line_no_space
"""Auto-ported test: NetrcTestCase::test_comment_at_end_of_machine_line_no_space (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_comment('            machine foo.domain.com login bar password pass #comment\n            machine bar.domain.com login foo password pass\n            ')
print("NetrcTestCase::test_comment_at_end_of_machine_line_no_space: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_comment_at_end_of_machine_line_no_space: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_comment_at_end_of_machine_line_pass_has_hash.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_comment_at_end_of_machine_line_pass_has_hash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_comment_at_end_of_machine_line_pass_has_hash"
# subject = "cpython.test_netrc.NetrcTestCase.test_comment_at_end_of_machine_line_pass_has_hash"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_comment_at_end_of_machine_line_pass_has_hash
"""Auto-ported test: NetrcTestCase::test_comment_at_end_of_machine_line_pass_has_hash (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_comment('            machine foo.domain.com login bar password #pass #comment\n            machine bar.domain.com login foo password pass\n            ', '#pass')
print("NetrcTestCase::test_comment_at_end_of_machine_line_pass_has_hash: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_comment_at_end_of_machine_line_pass_has_hash: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_comment_before_machine_line.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_comment_before_machine_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_comment_before_machine_line"
# subject = "cpython.test_netrc.NetrcTestCase.test_comment_before_machine_line"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_comment_before_machine_line
"""Auto-ported test: NetrcTestCase::test_comment_before_machine_line (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_comment('            # comment\n            machine foo.domain.com login bar password pass\n            machine bar.domain.com login foo password pass\n            ')
print("NetrcTestCase::test_comment_before_machine_line: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_comment_before_machine_line: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_comment_before_machine_line_hash_only.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_comment_before_machine_line_hash_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_comment_before_machine_line_hash_only"
# subject = "cpython.test_netrc.NetrcTestCase.test_comment_before_machine_line_hash_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_comment_before_machine_line_hash_only
"""Auto-ported test: NetrcTestCase::test_comment_before_machine_line_hash_only (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_comment('            #\n            machine foo.domain.com login bar password pass\n            machine bar.domain.com login foo password pass\n            ')
print("NetrcTestCase::test_comment_before_machine_line_hash_only: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_comment_before_machine_line_hash_only: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_comment_before_machine_line_no_space.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_comment_before_machine_line_no_space() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_comment_before_machine_line_no_space"
# subject = "cpython.test_netrc.NetrcTestCase.test_comment_before_machine_line_no_space"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_comment_before_machine_line_no_space
"""Auto-ported test: NetrcTestCase::test_comment_before_machine_line_no_space (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_comment('            #comment\n            machine foo.domain.com login bar password pass\n            machine bar.domain.com login foo password pass\n            ')
print("NetrcTestCase::test_comment_before_machine_line_no_space: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_comment_before_machine_line_no_space: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_invalid_tokens.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_invalid_tokens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_invalid_tokens"
# subject = "cpython.test_netrc.NetrcTestCase.test_invalid_tokens"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_invalid_tokens
"""Auto-ported test: NetrcTestCase::test_invalid_tokens (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
data = ('invalid host.domain.com', 'machine host.domain.com invalid', 'machine host.domain.com login log password pass account acct invalid', 'default host.domain.com invalid', 'default host.domain.com login log password pass account acct invalid')
for item in data:

    try:
        make_nrc(item)
        raise AssertionError('expected netrc.NetrcParseError')
    except netrc.NetrcParseError:
        pass
print("NetrcTestCase::test_invalid_tokens: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_invalid_tokens: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_macros.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_macros() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_macros"
# subject = "cpython.test_netrc.NetrcTestCase.test_macros"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_macros
"""Auto-ported test: NetrcTestCase::test_macros (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
data = '            macdef macro1\n            line1\n            line2\n\n            macdef macro2\n            line3\n            line4\n\n        '
nrc = make_nrc(data)

assert nrc.macros == {'macro1': ['line1\n', 'line2\n'], 'macro2': ['line3\n', 'line4\n']}

try:
    make_nrc(data.rstrip(' ')[:-1])
    raise AssertionError('expected netrc.NetrcParseError')
except netrc.NetrcParseError:
    pass
print("NetrcTestCase::test_macros: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_macros: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_optional_tokens.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_optional_tokens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_optional_tokens"
# subject = "cpython.test_netrc.NetrcTestCase.test_optional_tokens"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_optional_tokens
"""Auto-ported test: NetrcTestCase::test_optional_tokens (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
data = ('machine host.domain.com', 'machine host.domain.com login', 'machine host.domain.com account', 'machine host.domain.com password', 'machine host.domain.com login "" account', 'machine host.domain.com login "" password', 'machine host.domain.com account "" password')
for item in data:
    nrc = make_nrc(item)

    assert nrc.hosts['host.domain.com'] == ('', '', '')
data = ('default', 'default login', 'default account', 'default password', 'default login "" account', 'default login "" password', 'default account "" password')
for item in data:
    nrc = make_nrc(item)

    assert nrc.hosts['default'] == ('', '', '')
print("NetrcTestCase::test_optional_tokens: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_optional_tokens: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_token_value_escape.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_token_value_escape() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_token_value_escape"
# subject = "cpython.test_netrc.NetrcTestCase.test_token_value_escape"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_token_value_escape
"""Auto-ported test: NetrcTestCase::test_token_value_escape (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_token_x('            machine host.domain.com login \\"log password pass account acct\n            ', 'login', '"log')
_test_token_x('            machine host.domain.com login "\\"log" password pass account acct\n            ', 'login', '"log')
_test_token_x('            machine host.domain.com login log password pass account \\"acct\n            ', 'account', '"acct')
_test_token_x('            machine host.domain.com login log password pass account "\\"acct"\n            ', 'account', '"acct')
_test_token_x('            machine host.domain.com login log password \\"pass account acct\n            ', 'password', '"pass')
_test_token_x('            machine host.domain.com login log password "\\"pass" account acct\n            ', 'password', '"pass')
print("NetrcTestCase::test_token_value_escape: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_token_value_escape: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_token_value_internal_hash.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_token_value_internal_hash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_token_value_internal_hash"
# subject = "cpython.test_netrc.NetrcTestCase.test_token_value_internal_hash"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_token_value_internal_hash
"""Auto-ported test: NetrcTestCase::test_token_value_internal_hash (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_token_x('            machine host.domain.com login lo#g password pass account acct\n            ', 'login', 'lo#g')
_test_token_x('            machine host.domain.com login log password pass account ac#ct\n            ', 'account', 'ac#ct')
_test_token_x('            machine host.domain.com login log password pa#ss account acct\n            ', 'password', 'pa#ss')
print("NetrcTestCase::test_token_value_internal_hash: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_token_value_internal_hash: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_token_value_leading_hash.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_token_value_leading_hash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_token_value_leading_hash"
# subject = "cpython.test_netrc.NetrcTestCase.test_token_value_leading_hash"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_token_value_leading_hash
"""Auto-ported test: NetrcTestCase::test_token_value_leading_hash (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_token_x('            machine host.domain.com login #log password pass account acct\n            ', 'login', '#log')
_test_token_x('            machine host.domain.com login log password pass account #acct\n            ', 'account', '#acct')
_test_token_x('            machine host.domain.com login log password #pass account acct\n            ', 'password', '#pass')
print("NetrcTestCase::test_token_value_leading_hash: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_token_value_leading_hash: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_token_value_non_ascii.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_token_value_non_ascii() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_token_value_non_ascii"
# subject = "cpython.test_netrc.NetrcTestCase.test_token_value_non_ascii"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_token_value_non_ascii
"""Auto-ported test: NetrcTestCase::test_token_value_non_ascii (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_token_x('            machine host.domain.com login ¡¢ password pass account acct\n            ', 'login', '¡¢')
_test_token_x('            machine host.domain.com login log password pass account ¡¢\n            ', 'account', '¡¢')
_test_token_x('            machine host.domain.com login log password ¡¢ account acct\n            ', 'password', '¡¢')
print("NetrcTestCase::test_token_value_non_ascii: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_token_value_non_ascii: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_token_value_quotes.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_token_value_quotes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_token_value_quotes"
# subject = "cpython.test_netrc.NetrcTestCase.test_token_value_quotes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_token_value_quotes
"""Auto-ported test: NetrcTestCase::test_token_value_quotes (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_token_x('            machine host.domain.com login "log" password pass account acct\n            ', 'login', 'log')
_test_token_x('            machine host.domain.com login log password pass account "acct"\n            ', 'account', 'acct')
_test_token_x('            machine host.domain.com login log password "pass" account acct\n            ', 'password', 'pass')
print("NetrcTestCase::test_token_value_quotes: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_token_value_quotes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_token_value_trailing_hash.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_token_value_trailing_hash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_token_value_trailing_hash"
# subject = "cpython.test_netrc.NetrcTestCase.test_token_value_trailing_hash"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_token_value_trailing_hash
"""Auto-ported test: NetrcTestCase::test_token_value_trailing_hash (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_token_x('            machine host.domain.com login log# password pass account acct\n            ', 'login', 'log#')
_test_token_x('            machine host.domain.com login log password pass account acct#\n            ', 'account', 'acct#')
_test_token_x('            machine host.domain.com login log password pass# account acct\n            ', 'password', 'pass#')
print("NetrcTestCase::test_token_value_trailing_hash: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_token_value_trailing_hash: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_token_value_whitespace.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_token_value_whitespace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_token_value_whitespace"
# subject = "cpython.test_netrc.NetrcTestCase.test_token_value_whitespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_token_value_whitespace
"""Auto-ported test: NetrcTestCase::test_token_value_whitespace (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
_test_token_x('            machine host.domain.com login "lo g" password pass account acct\n            ', 'login', 'lo g')
_test_token_x('            machine host.domain.com login log password "pas s" account acct\n            ', 'password', 'pas s')
_test_token_x('            machine host.domain.com login log password pass account "acc t"\n            ', 'account', 'acc t')
print("NetrcTestCase::test_token_value_whitespace: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_token_value_whitespace: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_toplevel_non_ordered_tokens.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_toplevel_non_ordered_tokens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_toplevel_non_ordered_tokens"
# subject = "cpython.test_netrc.NetrcTestCase.test_toplevel_non_ordered_tokens"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_toplevel_non_ordered_tokens
"""Auto-ported test: NetrcTestCase::test_toplevel_non_ordered_tokens (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
nrc = make_nrc('            machine host.domain.com password pass1 login log1 account acct1\n            default login log2 password pass2 account acct2\n            ')

assert nrc.hosts['host.domain.com'] == ('log1', 'acct1', 'pass1')

assert nrc.hosts['default'] == ('log2', 'acct2', 'pass2')
print("NetrcTestCase::test_toplevel_non_ordered_tokens: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_toplevel_non_ordered_tokens: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/netrc/netrc_test_case__test_toplevel_tokens.py`.
#[test]
fn test_gen_behavior_std_libs_netrc_netrc_test_case__test_toplevel_tokens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "behavior"
# case = "netrc_test_case__test_toplevel_tokens"
# subject = "cpython.test_netrc.NetrcTestCase.test_toplevel_tokens"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_netrc.py::NetrcTestCase::test_toplevel_tokens
"""Auto-ported test: NetrcTestCase::test_toplevel_tokens (CPython 3.12 oracle)."""


import netrc, os, unittest, sys, textwrap
from test.support import os_helper


try:
    import pwd
except ImportError:
    pwd = None

temp_filename = os_helper.TESTFN


# --- test body ---
def _test_comment(nrc, passwd='pass'):
    nrc = make_nrc(nrc)

    assert nrc.hosts['foo.domain.com'] == ('bar', '', passwd)

    assert nrc.hosts['bar.domain.com'] == ('foo', '', 'pass')

def _test_token_x(nrc, token, value):
    nrc = make_nrc(nrc)
    if token == 'login':

        assert nrc.hosts['host.domain.com'] == (value, 'acct', 'pass')
    elif token == 'account':

        assert nrc.hosts['host.domain.com'] == ('log', value, 'pass')
    elif token == 'password':

        assert nrc.hosts['host.domain.com'] == ('log', 'acct', value)

def make_nrc(test_data):
    test_data = textwrap.dedent(test_data)
    mode = 'w'
    if sys.platform != 'cygwin':
        mode += 't'
    with open(temp_filename, mode, encoding='utf-8') as fp:
        fp.write(test_data)
    try:
        nrc = netrc.netrc(temp_filename)
    finally:
        os.unlink(temp_filename)
    return nrc
nrc = make_nrc('            machine host.domain.com login log1 password pass1 account acct1\n            default login log2 password pass2 account acct2\n            ')

assert nrc.hosts['host.domain.com'] == ('log1', 'acct1', 'pass1')

assert nrc.hosts['default'] == ('log2', 'acct2', 'pass2')
print("NetrcTestCase::test_toplevel_tokens: ok")
"###);
    assert_output(&out, r###"NetrcTestCase::test_toplevel_tokens: ok
"###);
}
