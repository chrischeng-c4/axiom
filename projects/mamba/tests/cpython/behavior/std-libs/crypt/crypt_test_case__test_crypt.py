# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "crypt"
# dimension = "behavior"
# case = "crypt_test_case__test_crypt"
# subject = "cpython.test_crypt.CryptTestCase.test_crypt"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_crypt.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_crypt.py::CryptTestCase::test_crypt
"""Auto-ported test: CryptTestCase::test_crypt (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import check_sanitizer, warnings_helper


try:
    if check_sanitizer(address=True, memory=True):
        raise unittest.SkipTest('The crypt module SEGFAULTs on ASAN/MSAN builds')
    crypt = warnings_helper.import_deprecated('crypt')
    IMPORT_ERROR = None
except ImportError as ex:
    if sys.platform != 'win32':
        raise unittest.SkipTest(str(ex))
    crypt = None
    IMPORT_ERROR = str(ex)


# --- test body ---
cr = crypt.crypt('mypassword')
cr2 = crypt.crypt('mypassword', cr)

assert cr2 == cr
cr = crypt.crypt('mypassword', 'ab')
if cr is not None:
    cr2 = crypt.crypt('mypassword', cr)

    assert cr2 == cr
print("CryptTestCase::test_crypt: ok")
