# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "non_subscriptable_type_raises"
# subject = "typing"
# kind = "mechanical"
# xfail = "mamba does not raise on subscripting a non-subscriptable builtin type (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing: non_subscriptable_type_raises (errors)."""
import typing

_raised = False
try:
    int['x']
except TypeError:
    _raised = True
assert _raised, "non_subscriptable_type_raises: expected TypeError"
print("non_subscriptable_type_raises OK")
