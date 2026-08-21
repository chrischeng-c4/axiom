# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "classmethod_alternate_constructor"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba classmethod returning cls() returns None (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.dataclass: a classmethod can act as an alternate constructor returning cls(...)"""
from dataclasses import dataclass


@dataclass
class Box:
    v: int

    @classmethod
    def doubled(cls, n):
        return cls(n * 2)


assert Box.doubled(3).v == 6
print("classmethod_alternate_constructor OK")
