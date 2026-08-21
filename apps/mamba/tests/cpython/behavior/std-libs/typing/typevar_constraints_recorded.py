# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "typevar_constraints_recorded"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.TypeVar: TypeVar('T', int, str) records the constraint set (int, str) on __constraints__ and leaves __bound__ unset"""
import typing

T = typing.TypeVar("T", int, str)
assert T.__constraints__ == (int, str), "TypeVar('T', int, str).__constraints__"
assert T.__bound__ is None, "a constrained TypeVar has no bound"
print("typevar_constraints_recorded OK")
