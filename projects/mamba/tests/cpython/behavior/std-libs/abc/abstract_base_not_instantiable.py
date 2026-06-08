# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "abstract_base_not_instantiable"
# subject = "abc.ABC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABC: instantiating an ABC subclass with an unimplemented abstractmethod raises TypeError"""
import abc


class Base(abc.ABC):
    @abc.abstractmethod
    def do(self) -> int: ...


_raised = False
try:
    Base()
except TypeError:
    _raised = True
assert _raised, "abstract base raises TypeError on instantiation"

print("abstract_base_not_instantiable OK")
