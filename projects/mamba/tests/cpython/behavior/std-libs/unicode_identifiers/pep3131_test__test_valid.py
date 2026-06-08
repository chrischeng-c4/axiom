# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode_identifiers"
# dimension = "behavior"
# case = "pep3131_test__test_valid"
# subject = "cpython.test_unicode_identifiers.PEP3131Test.test_valid"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode_identifiers.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicode_identifiers.py::PEP3131Test::test_valid
"""Auto-ported test: PEP3131Test::test_valid (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class T:
    ä = 1
    μ = 2
    蟒 = 3
    x󠄀 = 4

assert getattr(T, 'ä') == 1

assert getattr(T, 'μ') == 2

assert getattr(T, '蟒') == 3

assert getattr(T, 'x󠄀') == 4
print("PEP3131Test::test_valid: ok")
