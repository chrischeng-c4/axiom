# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertDictContainsSubset__subset_as_Mapping_wrong"
# subject = "unittest.case.TestCase.assertDictContainsSubset(subset: Mapping)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed subset"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed subset
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertDictContainsSubset(subset: Mapping); call it with the wrong type.

typeshed contract: subset is Mapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertDictContainsSubset(_W(), None)  # subset: Mapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
