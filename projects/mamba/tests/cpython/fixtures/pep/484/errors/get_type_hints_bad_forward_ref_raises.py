# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "get_type_hints_bad_forward_ref_raises"
# subject = "typing.get_type_hints"
# kind = "mechanical"
# xfail = "mamba does not resolve/raise on an undefined forward ref in get_type_hints (project_mamba_function_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_type_hints: get_type_hints_bad_forward_ref_raises (errors)."""
import typing

def _with_bad_ref(x: "NoSuchType") -> int:
    return 1


_raised = False
try:
    typing.get_type_hints(_with_bad_ref)
except NameError:
    _raised = True
assert _raised, "get_type_hints_bad_forward_ref_raises: expected NameError"
print("get_type_hints_bad_forward_ref_raises OK")
