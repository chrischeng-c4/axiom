# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "partial_concrete_still_abstract"
# subject = "abc.ABC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABC: a subclass that does not override the abstractmethod is still abstract and not instantiable"""
import abc


class Base(abc.ABC):
    @abc.abstractmethod
    def do(self) -> int: ...


class PartialConcrete(Base):
    pass  # does not implement do()


# The abstract method is inherited, so the subclass is still abstract.
assert "do" in PartialConcrete.__abstractmethods__, "do still abstract in subclass"

_raised = False
try:
    PartialConcrete()
except TypeError:
    _raised = True
assert _raised, "partial concrete subclass still raises TypeError"

print("partial_concrete_still_abstract OK")
