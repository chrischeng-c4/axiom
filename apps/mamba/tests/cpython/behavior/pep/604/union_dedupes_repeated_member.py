# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
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

# A union of a single repeated member collapses back to the bare type.
assert (int | int) is int
# A repeated member inside a larger union is deduplicated, preserving order.
assert (int | str | int).__args__ == (int, str)

print("union_dedupes_repeated_member OK")
