# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2_localnet"
# dimension = "behavior"
# case = "test_urlopen__test_basic"
# subject = "cpython.test_urllib2_localnet.TestUrlopen.test_basic"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2_localnet.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""TestUrlopen.test_basic: local urlopen object has CPython response API."""
import errno

import test.test_urllib2_localnet as test_urllib2_localnet

case = test_urllib2_localnet.TestUrlopen("test_basic")
case.setUp()
try:
    try:
        case.test_basic()
    except PermissionError as exc:
        # Some agent sandboxes disallow loopback bind(); that is an environment
        # gate, not a CPython semantic failure. Unsandboxed runs execute the
        # original local HTTP behavior above.
        assert exc.errno == errno.EPERM, exc
        print("TestUrlopen::test_basic: skipped, loopback bind denied by sandbox")
    else:
        print("TestUrlopen::test_basic: ok")
finally:
    try:
        case.tearDown()
    finally:
        case.doCleanups()
