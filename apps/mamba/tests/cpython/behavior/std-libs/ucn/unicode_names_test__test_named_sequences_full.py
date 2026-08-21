# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ucn"
# dimension = "behavior"
# case = "unicode_names_test__test_named_sequences_full"
# subject = "cpython.test_ucn.UnicodeNamesTest.test_named_sequences_full"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ucn.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: UnicodeNamesTest::test_named_sequences_full (CPython 3.12 oracle)."""

import unittest
from test import support
from test.test_ucn import UnicodeNamesTest


def unavailable_urlresource(*args, **kwargs):
    raise OSError("deterministic url resource unavailable")


original = support.open_urlresource
support.open_urlresource = unavailable_urlresource
try:
    case = UnicodeNamesTest("test_named_sequences_full")
    result = unittest.TestResult()
    case.run(result)
finally:
    support.open_urlresource = original

assert result.wasSuccessful(), result
assert len(result.skipped) == 1, result.skipped
assert result.skipped[0][0] is case
reason = result.skipped[0][1]
assert reason.startswith("Could not retrieve http://www.pythontest.net/unicode/")
assert "NamedSequences.txt" in reason
assert "deterministic url resource unavailable" in reason

print("UnicodeNamesTest::test_named_sequences_full resource boundary: ok")
