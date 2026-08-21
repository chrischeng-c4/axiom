# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "isabstract_true_only_for_unimplemented_abc"
# subject = "inspect.isabstract"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isabstract: isabstract is True only for an ABC with an unimplemented abstractmethod; False for a concrete subclass, an instance, and a builtin"""
import inspect
from abc import ABCMeta, abstractmethod

class AbstractBase(metaclass=ABCMeta):
    @abstractmethod
    def foo(self):
        pass

class Concrete(AbstractBase):
    def foo(self):
        pass

assert inspect.isabstract(AbstractBase), "abstract class"
assert not inspect.isabstract(Concrete), "concrete subclass"
assert not inspect.isabstract(Concrete()), "instance is not abstract"
assert not inspect.isabstract(int), "builtin not abstract"

print("isabstract_true_only_for_unimplemented_abc OK")
