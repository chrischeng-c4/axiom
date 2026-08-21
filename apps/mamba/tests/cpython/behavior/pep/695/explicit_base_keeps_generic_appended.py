# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "explicit_base_keeps_generic_appended"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = "Child.__bases__ returns None on mamba so the (Base, Generic) ordering can't be checked (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: an explicit base is preserved with Generic appended after it; keyword args and **dict expansion flow to __init_subclass__ (Child[T](Base, a=1, b=2, **extra))"""
from typing import Generic


# An explicit base is preserved, with Generic appended after it.
class Base:
    def __init_subclass__(cls, **kwargs):
        cls.kwargs = kwargs


# Keyword args and **dict expansion flow to __init_subclass__ as usual.
extra = {"c": 3}
class Child[T](Base, a=1, b=2, **extra):
    pass


assert Child.__bases__ == (Base, Generic)
assert Child.kwargs == {"a": 1, "b": 2, "c": 3}

print("explicit_base_keeps_generic_appended OK")
