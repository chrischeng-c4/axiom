# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "unicode_misc_test__test_disallow_instantiation"
# subject = "cpython.test_unicodedata.UnicodeMiscTest.test_disallow_instantiation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: UnicodeMiscTest::test_disallow_instantiation (CPython 3.12 oracle)."""

import unicodedata


try:
    unicodedata.UCD()
except TypeError as exc:
    assert str(exc) == "cannot create 'unicodedata.UCD' instances", str(exc)
else:
    raise AssertionError("unicodedata.UCD() should reject direct instantiation")

assert isinstance(unicodedata.ucd_3_2_0, unicodedata.UCD)

print("UnicodeMiscTest::test_disallow_instantiation: ok")
