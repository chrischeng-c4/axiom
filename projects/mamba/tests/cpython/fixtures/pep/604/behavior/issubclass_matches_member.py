# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "behavior"
# case = "issubclass_matches_member"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: issubclass(int, int | str) is True; the union acts as the second arg to issubclass"""
import types

u = int | str
assert issubclass(int, u) is True
assert issubclass(str, u) is True
assert issubclass(float, u) is False
# A subclass of a member still matches (bool is a subclass of int).
assert issubclass(bool, u) is True

print("issubclass_matches_member OK")
