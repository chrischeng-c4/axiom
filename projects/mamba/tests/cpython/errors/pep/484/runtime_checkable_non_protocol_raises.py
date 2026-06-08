# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "runtime_checkable_non_protocol_raises"
# subject = "typing.runtime_checkable"
# kind = "mechanical"
# xfail = "mamba does not guard @runtime_checkable to Protocol classes (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.runtime_checkable: runtime_checkable_non_protocol_raises (errors)."""
import typing

class _PlainClass:
    pass


_raised = False
try:
    typing.runtime_checkable(_PlainClass)
except TypeError:
    _raised = True
assert _raised, "runtime_checkable_non_protocol_raises: expected TypeError"
print("runtime_checkable_non_protocol_raises OK")
