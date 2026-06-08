# /// script
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
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_netrc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
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
