# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "classvar_in_annotations"
# subject = "typing.ClassVar"
# kind = "semantic"
# xfail = "class __annotations__ is an undefined name on mamba; ClassVar declaration machinery diverges. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.ClassVar: a `count: ClassVar[int] = 0` declaration is recorded in __annotations__ and the ClassVar attribute is shared (mutating it via the class is visible to all instances)"""
from typing import ClassVar


class Counter:
    count: ClassVar[int] = 0

    def __init__(self):
        Counter.count += 1


assert "count" in Counter.__annotations__, Counter.__annotations__
Counter.count = 0
c1 = Counter()
c2 = Counter()
# The ClassVar is class-shared state: both instances bumped the one counter.
assert Counter.count == 2, Counter.count
assert c1.count == 2 and c2.count == 2, (c1.count, c2.count)
print("classvar_in_annotations OK")
