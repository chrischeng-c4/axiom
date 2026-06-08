# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "typevar_bound_recorded"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.TypeVar: TypeVar('T', bound=int) records the upper bound on __bound__ and leaves __constraints__ empty"""
import typing

T = typing.TypeVar("T", bound=int)
assert T.__bound__ is int, "TypeVar(bound=int).__bound__ should be int"
assert T.__constraints__ == (), "a bounded TypeVar has no constraints"
print("typevar_bound_recorded OK")
