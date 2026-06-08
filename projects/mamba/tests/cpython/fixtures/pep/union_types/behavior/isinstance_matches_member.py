# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "isinstance_matches_member"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "isinstance(x, int | str) raises TypeError on mamba (None second arg; project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: isinstance(x, int | str) is True for an int or a str member and False for a non-member (float)"""
import types

u = int | str
assert isinstance(1, u) is True
assert isinstance("a", u) is True
assert isinstance(1.5, u) is False

print("isinstance_matches_member OK")
