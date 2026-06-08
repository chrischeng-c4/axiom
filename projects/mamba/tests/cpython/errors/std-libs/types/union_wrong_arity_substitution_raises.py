# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "errors"
# case = "union_wrong_arity_substitution_raises"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: substituting the wrong number of parameters into a parameterized union (int | T)[int, str] raises TypeError"""
import types  # noqa: F401
import typing

T = typing.TypeVar("T")
partial = int | T

_raised = False
try:
    partial[int, str]
except TypeError:
    _raised = True
assert _raised, "union_wrong_arity_substitution_raises: expected TypeError"

print("union_wrong_arity_substitution_raises OK")
