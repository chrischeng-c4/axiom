# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "abcmeta_explicit_metaclass"
# subject = "abc.ABCMeta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABCMeta: using ABCMeta directly as metaclass gives the same abstract-enforcement and isinstance behavior as inheriting ABC"""
import abc


class Animal(metaclass=abc.ABCMeta):
    @abc.abstractmethod
    def speak(self) -> str: ...


# Abstract enforcement applies just as with ABC inheritance.
_raised = False
try:
    Animal()
except TypeError:
    _raised = True
assert _raised, "ABCMeta-based abstract class is not instantiable"


class Dog(Animal):
    def speak(self) -> str:
        return "woof"


assert Dog().speak() == "woof", "concrete ABCMeta subclass works"
assert isinstance(Dog(), Animal), "Dog instance isinstance Animal"
assert issubclass(Dog, Animal), "Dog issubclass Animal"

print("abcmeta_explicit_metaclass OK")
