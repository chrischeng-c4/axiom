# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "init_positional_and_keyword"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize __init__ from annotated fields (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.dataclass: the generated __init__ assigns fields positionally, by keyword, and applies declared defaults"""
from dataclasses import dataclass


@dataclass
class Point:
    x: int
    y: int = 0


p = Point(1, 2)
assert (p.x, p.y) == (1, 2)
assert Point(5).y == 0
assert Point(y=9, x=1) == Point(1, 9)
print("init_positional_and_keyword OK")
