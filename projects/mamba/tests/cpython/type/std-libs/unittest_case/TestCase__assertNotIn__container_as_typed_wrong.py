# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertNotIn__container_as_typed_wrong"
# subject = "unittest.case.TestCase.assertNotIn(container: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertNotIn(container: typed); call it with the wrong type.

typeshed contract: container is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertNotIn(None, _W())  # container: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
