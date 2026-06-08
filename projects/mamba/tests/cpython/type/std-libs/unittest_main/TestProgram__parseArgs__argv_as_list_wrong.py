# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_main"
# dimension = "type"
# case = "TestProgram__parseArgs__argv_as_list_wrong"
# subject = "unittest.main.TestProgram.parseArgs(argv: list)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed argv"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/main.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed argv
# mamba-strict-type: TypeError
"""Type wall: unittest.main.TestProgram.parseArgs(argv: list); call it with the wrong type.

typeshed contract: argv is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.main import TestProgram
obj = object.__new__(TestProgram)
try:
    obj.parseArgs(12345)  # argv: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
