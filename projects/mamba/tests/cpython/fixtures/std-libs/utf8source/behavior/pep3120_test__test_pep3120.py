# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "utf8source"
# dimension = "behavior"
# case = "pep3120_test__test_pep3120"
# subject = "cpython.test_utf8source.PEP3120Test.test_pep3120"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_utf8source.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_utf8source.py::PEP3120Test::test_pep3120
"""Auto-ported test: PEP3120Test::test_pep3120 (CPython 3.12 oracle)."""


import unittest


# --- test body ---

assert 'Питон'.encode('utf-8') == b'\xd0\x9f\xd0\xb8\xd1\x82\xd0\xbe\xd0\xbd'

assert '\\П'.encode('utf-8') == b'\\\xd0\x9f'
print("PEP3120Test::test_pep3120: ok")
