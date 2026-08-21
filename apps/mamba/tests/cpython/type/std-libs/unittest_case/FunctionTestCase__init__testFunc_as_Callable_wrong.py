# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "FunctionTestCase__init__testFunc_as_Callable_wrong"
# subject = "unittest.case.FunctionTestCase.__init__(testFunc: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.FunctionTestCase.__init__(testFunc: Callable); call it with the wrong type.

typeshed contract: testFunc is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import FunctionTestCase
try:
    FunctionTestCase(_W())  # testFunc: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
