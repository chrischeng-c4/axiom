# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "all_abstract_methods_required"
# subject = "abc.abstractmethod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.abstractmethod: with multiple abstract methods, every one must be implemented before the class is instantiable"""
import abc


class Multi(abc.ABC):
    @abc.abstractmethod
    def m1(self) -> int: ...
    @abc.abstractmethod
    def m2(self) -> int: ...


# Implementing only one abstract method is not enough.
class ImplOne(Multi):
    def m1(self) -> int:
        return 1


_raised = False
try:
    ImplOne()
except TypeError:
    _raised = True
assert _raised, "missing m2 keeps the class abstract"

# Implementing both makes it concrete.
class ImplBoth(Multi):
    def m1(self) -> int:
        return 1
    def m2(self) -> int:
        return 2


both = ImplBoth()
assert both.m1() == 1 and both.m2() == 2, "both abstract methods implemented"

print("all_abstract_methods_required OK")
