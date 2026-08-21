# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "union_args_tuple"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: (int | str | bytes).__args__ holds exactly the member types {int, str, bytes}"""
import types

args = (int | str | bytes).__args__
assert isinstance(args, tuple)
assert set(args) == {int, str, bytes}, args

print("union_args_tuple OK")
