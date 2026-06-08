# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "create_autospec_wrong_signature_raises"
# subject = "unittest.mock.create_autospec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.create_autospec: create_autospec(func) enforces the wrapped signature: calling it with a missing required argument raises TypeError"""
from unittest.mock import create_autospec


def f(a, b):
    return a + b


af = create_autospec(f)
af(1, 2)  # valid call is accepted
_raised = False
try:
    af(1)  # missing required argument b
except TypeError:
    _raised = True
assert _raised, "autospec must enforce the wrapped signature"
print("create_autospec_wrong_signature_raises OK")
