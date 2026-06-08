# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_main"
# dimension = "type"
# case = "TestProgram__createTests__from_discovery_as_bool_wrong"
# subject = "unittest.main.TestProgram.createTests(from_discovery: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed from_discovery"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/main.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed from_discovery
# mamba-strict-type: TypeError
"""Type wall: unittest.main.TestProgram.createTests(from_discovery: bool); call it with the wrong type.

typeshed contract: from_discovery is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.main import TestProgram
obj = object.__new__(TestProgram)
try:
    obj.createTests("not_a_bool")  # from_discovery: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
