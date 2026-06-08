# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "full_concrete_instantiable"
# subject = "abc.ABC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABC: a subclass that implements the abstractmethod is concrete, instantiable, and has an empty __abstractmethods__"""
import abc


class Base(abc.ABC):
    @abc.abstractmethod
    def do(self) -> int: ...


class FullConcrete(Base):
    def do(self) -> int:
        return 42


assert FullConcrete().do() == 42, "concrete do() returns 42"
assert len(FullConcrete.__abstractmethods__) == 0, "concrete class has no abstract methods"

print("full_concrete_instantiable OK")
