# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "errors"
# case = "non_generic_subscript_raises"
# subject = "int"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""int: non_generic_subscript_raises (errors)."""
import typing  # noqa: F401

_raised = False
try:
    int["x"]
except TypeError:
    _raised = True
assert _raised, "non_generic_subscript_raises: expected TypeError"
print("non_generic_subscript_raises OK")
