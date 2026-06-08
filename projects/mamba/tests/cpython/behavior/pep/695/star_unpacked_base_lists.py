# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "star_unpacked_base_lists"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = "Empty/Starred.__bases__ return None on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: star-unpacked base lists work for generic classes: class Empty[T](*()) yields just (Generic,) and class Starred[T](*[Base]) yields (Base, Generic)"""
from typing import Generic


class Base:
    pass


# Star-unpacked base lists work too: an empty one yields just Generic.
class Empty[T](*()):
    pass


assert Empty.__bases__ == (Generic,)

bases = [Base]
class Starred[T](*bases):
    pass


assert Starred.__bases__ == (Base, Generic)

print("star_unpacked_base_lists OK")
