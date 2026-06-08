# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertNotAlmostEqual__first_as_SupportsSub_wrong"
# subject = "unittest.case.TestCase.assertNotAlmostEqual(first: SupportsSub)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed first"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed first
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertNotAlmostEqual(first: SupportsSub); call it with the wrong type.

typeshed contract: first is SupportsSub. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertNotAlmostEqual(_W(), None)  # first: SupportsSub <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
