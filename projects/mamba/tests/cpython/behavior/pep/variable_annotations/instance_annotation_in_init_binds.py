# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "instance_annotation_in_init_binds"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "class __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: class-body instance annotations (`x: float; y: float`) are recorded in the class __annotations__ while the values are bound per-instance in __init__"""


class Point:
    x: float
    y: float

    def __init__(self, x: float, y: float):
        self.x = x
        self.y = y


assert sorted(Point.__annotations__.keys()) == ["x", "y"], Point.__annotations__
# The annotations do not bind values; __init__ binds them per instance.
p = Point(1.0, 2.0)
assert p.x == 1.0 and p.y == 2.0, (p.x, p.y)
print("instance_annotation_in_init_binds OK")
