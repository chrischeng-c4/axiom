# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "issubclass_matches_member"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: issubclass(x, int | str) is True for a member or a member's subclass (bool <: int) and False for a non-member (list)"""
import types

u = int | str
assert issubclass(int, u) is True
assert issubclass(str, u) is True
# A subclass of a member still matches (bool is a subclass of int).
assert issubclass(bool, u) is True
assert issubclass(list, u) is False

print("issubclass_matches_member OK")
