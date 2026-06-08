# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "union_multi_member_isinstance"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str | float | bytes` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: a four-member union int | str | float | bytes isinstance-matches each member type and rejects a non-member (list)"""
import types

multi = int | str | float | bytes
assert isinstance(42, multi) is True
assert isinstance("hi", multi) is True
assert isinstance(3.14, multi) is True
assert isinstance(b"x", multi) is True
assert isinstance([1], multi) is False

print("union_multi_member_isinstance OK")
