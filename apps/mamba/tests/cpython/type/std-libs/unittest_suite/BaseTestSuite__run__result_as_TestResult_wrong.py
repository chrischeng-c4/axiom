# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_suite"
# dimension = "type"
# case = "BaseTestSuite__run__result_as_TestResult_wrong"
# subject = "unittest.suite.BaseTestSuite.run(result: TestResult)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/suite.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.suite.BaseTestSuite.run(result: TestResult); call it with the wrong type.

typeshed contract: result is TestResult. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.suite import BaseTestSuite
obj = object.__new__(BaseTestSuite)
try:
    obj.run(_W())  # result: TestResult <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
