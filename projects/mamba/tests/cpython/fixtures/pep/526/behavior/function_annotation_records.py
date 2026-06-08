# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "function_annotation_records"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "function __annotations__ returns None on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: function parameter and return annotations populate fn.__annotations__ with keys {'a', 'b', 'return'}"""


def fn(a: int, b: str) -> bool:
    return True


assert sorted(fn.__annotations__.keys()) == ["a", "b", "return"], fn.__annotations__
print("function_annotation_records OK")
