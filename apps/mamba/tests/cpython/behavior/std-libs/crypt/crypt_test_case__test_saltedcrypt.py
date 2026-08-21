# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "crypt"
# dimension = "behavior"
# case = "crypt_test_case__test_saltedcrypt"
# subject = "cpython.test_crypt.CryptTestCase.test_saltedcrypt"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_crypt.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_crypt.py::CryptTestCase::test_saltedcrypt
"""Auto-ported test: CryptTestCase::test_saltedcrypt (CPython 3.12 oracle)."""


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
for method in crypt.methods:
    cr = crypt.crypt('assword', method)

    assert len(cr) == method.total_size
    cr2 = crypt.crypt('assword', cr)

    assert cr2 == cr
    cr = crypt.crypt('assword', crypt.mksalt(method))

    assert len(cr) == method.total_size
print("CryptTestCase::test_saltedcrypt: ok")
