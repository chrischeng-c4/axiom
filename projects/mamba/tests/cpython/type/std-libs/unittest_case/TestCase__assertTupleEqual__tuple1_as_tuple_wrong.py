# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertTupleEqual__tuple1_as_tuple_wrong"
# subject = "unittest.case.TestCase.assertTupleEqual(tuple1: tuple)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertTupleEqual(tuple1: tuple); call it with the wrong type.

typeshed contract: tuple1 is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertTupleEqual(12345, None)  # tuple1: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
