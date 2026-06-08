# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "crypt"
# dimension = "behavior"
# case = "crypt_test_case__test_invalid_rounds"
# subject = "cpython.test_crypt.CryptTestCase.test_invalid_rounds"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_crypt.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_crypt.py::CryptTestCase::test_invalid_rounds
"""Auto-ported test: CryptTestCase::test_invalid_rounds (CPython 3.12 oracle)."""


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
for method in (crypt.METHOD_SHA256, crypt.METHOD_SHA512, crypt.METHOD_BLOWFISH):
    try:
        crypt.mksalt(method, rounds='4096')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        crypt.mksalt(method, rounds=4096.0)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    for rounds in (0, 1, -1, 1 << 999):
        try:
            crypt.mksalt(method, rounds=rounds)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass
try:
    crypt.mksalt(crypt.METHOD_BLOWFISH, rounds=1000)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
for method in (crypt.METHOD_CRYPT, crypt.METHOD_MD5):
    try:
        crypt.mksalt(method, rounds=4096)
        raise AssertionError('expected ValueError')
    except ValueError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('support', str(_aR_e))
print("CryptTestCase::test_invalid_rounds: ok")
