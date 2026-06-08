# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "union_in_function_annotation"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str | None` annotation evaluates to None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: a `X | Y | None` parameter annotation does not change runtime call behavior; the function dispatches on isinstance over the union"""
import types


def process(val: int | str | None) -> str:
    if val is None:
        return "none"
    return str(val)


assert process(42) == "42"
assert process("hi") == "hi"
assert process(None) == "none"

print("union_in_function_annotation OK")
