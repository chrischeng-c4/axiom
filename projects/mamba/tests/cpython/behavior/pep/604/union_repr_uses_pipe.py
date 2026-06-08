# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "behavior"
# case = "union_repr_uses_pipe"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: the repr of int | str renders the members joined by ' | ': 'int | str'"""
import types

assert repr(int | str) == "int | str"
assert str(int | str) == "int | str"
# A three-member union joins every member with ' | '.
assert repr(int | str | bytes) == "int | str | bytes"

print("union_repr_uses_pipe OK")
