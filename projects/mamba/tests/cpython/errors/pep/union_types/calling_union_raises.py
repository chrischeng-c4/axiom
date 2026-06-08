# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "errors"
# case = "calling_union_raises"
# subject = "types.UnionType"
# kind = "mechanical"
# xfail = "`int | str` returns None on mamba so calling it does not raise TypeError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: calling_union_raises (errors)."""
import types

_raised = False
try:
    (int | str)(1)
except TypeError:
    _raised = True
assert _raised, "calling_union_raises: expected TypeError"
print("calling_union_raises OK")
