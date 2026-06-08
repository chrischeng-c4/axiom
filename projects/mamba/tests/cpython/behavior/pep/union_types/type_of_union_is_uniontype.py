# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "type_of_union_is_uniontype"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | float` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: type(int | float) is types.UnionType"""
import types

ut = int | float
assert type(ut) is types.UnionType, repr(type(ut))

print("type_of_union_is_uniontype OK")
