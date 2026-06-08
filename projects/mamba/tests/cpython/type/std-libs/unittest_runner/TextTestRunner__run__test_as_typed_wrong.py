# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_runner"
# dimension = "type"
# case = "TextTestRunner__run__test_as_typed_wrong"
# subject = "unittest.runner.TextTestRunner.run(test: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/runner.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.runner.TextTestRunner.run(test: typed); call it with the wrong type.

typeshed contract: test is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.runner import TextTestRunner
obj = object.__new__(TextTestRunner)
try:
    obj.run(_W())  # test: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
