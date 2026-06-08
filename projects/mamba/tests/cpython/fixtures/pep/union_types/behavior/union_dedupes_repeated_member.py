# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "union_dedupes_repeated_member"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | int` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: a union of one type with itself collapses to that bare type: int | int is int"""
import types

dup = int | int
assert dup is int, repr(dup)

print("union_dedupes_repeated_member OK")
