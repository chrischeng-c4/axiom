# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode_identifiers"
# dimension = "behavior"
# case = "pep3131_test__test_non_bmp_normalized"
# subject = "cpython.test_unicode_identifiers.PEP3131Test.test_non_bmp_normalized"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode_identifiers.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicode_identifiers.py::PEP3131Test::test_non_bmp_normalized
"""Auto-ported test: PEP3131Test::test_non_bmp_normalized (CPython 3.12 oracle)."""


import unittest


# --- test body ---
Unicode = 1

assert 'Unicode' in dir()
print("PEP3131Test::test_non_bmp_normalized: ok")
