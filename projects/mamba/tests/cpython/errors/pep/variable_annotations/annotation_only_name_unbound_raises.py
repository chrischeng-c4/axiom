# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "errors"
# case = "annotation_only_name_unbound_raises"
# subject = "exec"
# kind = "mechanical"
# xfail = "mamba exec defers parsing and returns None silently instead of executing the body / raising NameError. See project_mamba_eval_silent_none_cross_type."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""exec: annotation_only_name_unbound_raises (errors)."""
import typing

_raised = False
try:
    exec('y: int\nprint(y)')
except NameError:
    _raised = True
assert _raised, "annotation_only_name_unbound_raises: expected NameError"
print("annotation_only_name_unbound_raises OK")
