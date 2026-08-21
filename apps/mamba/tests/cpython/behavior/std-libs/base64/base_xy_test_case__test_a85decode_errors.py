# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_a85decode_errors"
# subject = "cpython.test_base64.BaseXYTestCase.test_a85decode_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_a85decode_errors
"""Auto-ported test: BaseXYTestCase::test_a85decode_errors (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
illegal = (set(range(32)) | set(range(118, 256))) - set(b' \t\n\r\x0b')
for c in illegal:
    try:
        base64.a85decode(b'!!!!' + bytes([c]))
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    try:
        base64.a85decode(b'!!!!' + bytes([c]), adobe=False)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    try:
        base64.a85decode(b'<~!!!!' + bytes([c]) + b'~>', adobe=True)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

try:
    base64.a85decode(b'malformed', adobe=True)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.a85decode(b'<~still malformed', adobe=True)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.a85decode(b'<~~>')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.a85decode(b'<~~>', adobe=False)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
base64.a85decode(b'<~~>', adobe=True)

try:
    base64.a85decode(b'abcx', adobe=False)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.a85decode(b'abcdey', adobe=False)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.a85decode(b'a b\nc', adobe=False, ignorechars=b'')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.a85decode(b's', adobe=False)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.a85decode(b's8', adobe=False)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.a85decode(b's8W', adobe=False)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.a85decode(b's8W-', adobe=False)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.a85decode(b's8W-"', adobe=False)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    base64.a85decode(b'aaaay', foldspaces=True)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("BaseXYTestCase::test_a85decode_errors: ok")
