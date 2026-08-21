# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_loader"
# dimension = "type"
# case = "TestLoader__getTestCaseNames__testCaseClass_as_type_wrong"
# subject = "unittest.loader.TestLoader.getTestCaseNames(testCaseClass: type)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed testCaseClass"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/loader.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed testCaseClass
# mamba-strict-type: TypeError
"""Type wall: unittest.loader.TestLoader.getTestCaseNames(testCaseClass: type); call it with the wrong type.

typeshed contract: testCaseClass is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.loader import TestLoader
obj = object.__new__(TestLoader)
try:
    obj.getTestCaseNames(_W())  # testCaseClass: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
