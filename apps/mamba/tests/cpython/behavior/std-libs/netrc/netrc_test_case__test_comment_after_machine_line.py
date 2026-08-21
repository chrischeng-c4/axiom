# /// script
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
