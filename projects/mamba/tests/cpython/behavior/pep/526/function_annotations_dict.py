# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "function_annotations_dict"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "function __annotations__ returns None on mamba; subscripting it raises TypeError. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: fn.__annotations__ is a dict mapping each annotated name to its annotation object: {'x': int, 'y': int, 'return': int}"""


def annotated(x: int, y: int) -> int:
    return x + y


ann = annotated.__annotations__
assert type(ann).__name__ == "dict", type(ann).__name__
assert sorted(ann.keys()) == ["return", "x", "y"], sorted(ann.keys())
assert ann["x"] is int and ann["y"] is int and ann["return"] is int, ann
print("function_annotations_dict OK")
