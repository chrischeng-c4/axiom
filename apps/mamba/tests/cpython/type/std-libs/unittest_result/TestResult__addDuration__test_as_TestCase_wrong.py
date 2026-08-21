# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_result"
# dimension = "type"
# case = "TestResult__addDuration__test_as_TestCase_wrong"
# subject = "unittest.result.TestResult.addDuration(test: TestCase)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/result.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.result.TestResult.addDuration(test: TestCase); call it with the wrong type.

typeshed contract: test is TestCase. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.result import TestResult
obj = object.__new__(TestResult)
try:
    obj.addDuration(_W(), 0.0)  # test: TestCase <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
