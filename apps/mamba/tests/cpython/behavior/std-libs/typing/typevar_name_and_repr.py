# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "typevar_name_and_repr"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.TypeVar: a bare TypeVar('T') records its name in __name__ and has neither bound nor constraints"""
import typing

T = typing.TypeVar("T")
assert T.__name__ == "T", "TypeVar.__name__ should be 'T'"
assert T.__bound__ is None, "a bare TypeVar has no bound"
assert T.__constraints__ == (), "a bare TypeVar has no constraints"
print("typevar_name_and_repr OK")
