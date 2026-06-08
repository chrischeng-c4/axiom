# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "union_flattens_nested"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: nested unions flatten: (int | str) | float and int | (str | float) both contain float and share __args__"""
import types

u1 = (int | str) | float
u2 = int | (str | float)
assert isinstance(3.14, u1) is True
assert isinstance(3.14, u2) is True
# Both nestings flatten to the same set of members.
assert set(u1.__args__) == {int, str, float}
assert set(u2.__args__) == {int, str, float}

print("union_flattens_nested OK")
