# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "implicit_generic_base"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = "Box.__bases__ returns None on mamba (Generic base not implicitly added; probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: class Box[T]: with no explicit bases implicitly gains Generic as its base, records Generic[T] in get_original_bases, and exposes __parameters__"""
import types
from typing import Generic


# A `class X[T]:` with no explicit bases implicitly gains Generic as a base,
# while __orig_bases__ records the parameterized Generic[T].
class Box[T]:
    def __init__(self, value: T) -> None:
        self.value = value


assert Box.__bases__ == (Generic,)
t_param, = Box.__type_params__
assert types.get_original_bases(Box) == (Generic[t_param],)
assert Box.__parameters__ == (t_param,)
assert Box(42).value == 42

print("implicit_generic_base OK")
