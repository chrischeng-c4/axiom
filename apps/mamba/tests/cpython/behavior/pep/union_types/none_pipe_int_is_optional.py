# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "none_pipe_int_is_optional"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | None` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: int | None is the Optional idiom: None and an int member both isinstance-match, a str does not"""
import types

nullable = int | None
assert isinstance(None, nullable) is True
assert isinstance(42, nullable) is True
assert isinstance("hi", nullable) is False

print("none_pipe_int_is_optional OK")
