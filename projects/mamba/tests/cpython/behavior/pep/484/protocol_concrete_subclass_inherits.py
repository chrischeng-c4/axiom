# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "protocol_concrete_subclass_inherits"
# subject = "typing.Protocol"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Protocol: a concrete class may subclass a Protocol to inherit its interface and is then an ordinary class: Square(Drawable) implements draw() and Square().draw()=='square'"""
from typing import Protocol


class Drawable(Protocol):
    def draw(self) -> None: ...


# A concrete class may subclass a Protocol to inherit its interface.
class Square(Drawable):
    def draw(self):
        return "square"


assert Square().draw() == "square"

print("protocol_concrete_subclass_inherits OK")
