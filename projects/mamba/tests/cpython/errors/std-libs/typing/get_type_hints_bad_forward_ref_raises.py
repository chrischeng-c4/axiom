# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "errors"
# case = "get_type_hints_bad_forward_ref_raises"
# subject = "typing.get_type_hints"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.get_type_hints: get_type_hints on a function annotated with an unresolvable forward reference 'NoSuchType' raises NameError"""
import typing


def with_bad_hint(x: "NoSuchType") -> int:  # noqa: F821
    return x


_raised = False
try:
    typing.get_type_hints(with_bad_hint)
except NameError:
    _raised = True
assert _raised, "get_type_hints_bad_forward_ref_raises: expected NameError"
print("get_type_hints_bad_forward_ref_raises OK")
