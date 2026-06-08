# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_b85decode_errors"
# subject = "cpython.test_base64.BaseXYTestCase.test_b85decode_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_b85decode_errors
"""Auto-ported test: BaseXYTestCase::test_b85decode_errors (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
illegal = list(range(33)) + list(b'"\',./:[\\]') + list(range(128, 256))
for c in illegal:
    try:
        base64.b85decode(b'0000' + bytes([c]))
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

try:
    base64.b85decode(b'|')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.b85decode(b'|N')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.b85decode(b'|Ns')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.b85decode(b'|NsC')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.b85decode(b'|NsC1')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("BaseXYTestCase::test_b85decode_errors: ok")
