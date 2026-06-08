# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "errors"
# case = "union_with_non_type_value_raises"
# subject = "types.UnionType"
# kind = "mechanical"
# xfail = "`int | 42` returns None on mamba instead of raising TypeError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: union_with_non_type_value_raises (errors)."""
import types

_raised = False
try:
    int | 42
except TypeError:
    _raised = True
assert _raised, "union_with_non_type_value_raises: expected TypeError"
print("union_with_non_type_value_raises OK")
