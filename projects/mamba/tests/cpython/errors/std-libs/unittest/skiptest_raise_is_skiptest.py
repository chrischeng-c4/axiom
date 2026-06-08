# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "errors"
# case = "skiptest_raise_is_skiptest"
# subject = "unittest.SkipTest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.SkipTest: skiptest_raise_is_skiptest (errors)."""
import unittest

_raised = False
try:
    raise unittest.SkipTest('skipping')
except unittest.SkipTest:
    _raised = True
assert _raised, "skiptest_raise_is_skiptest: expected unittest.SkipTest"
print("skiptest_raise_is_skiptest OK")
