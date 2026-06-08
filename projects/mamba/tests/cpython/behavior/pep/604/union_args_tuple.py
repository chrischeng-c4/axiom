# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "behavior"
# case = "union_args_tuple"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: (int | str).__args__ is the ordered tuple (int, str) of the union members"""
import types

u = int | str
assert u.__args__ == (int, str)
# Order follows the operands, and a three-member union keeps all three.
assert (int | str | bytes).__args__ == (int, str, bytes)

print("union_args_tuple OK")
