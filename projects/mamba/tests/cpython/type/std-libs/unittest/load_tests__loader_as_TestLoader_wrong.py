# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "type"
# case = "load_tests__loader_as_TestLoader_wrong"
# subject = "unittest.load_tests(loader: TestLoader)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.load_tests(loader: TestLoader); call it with the wrong type.

typeshed contract: loader is TestLoader. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest import load_tests
try:
    load_tests(_W(), None, None)  # loader: TestLoader <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
