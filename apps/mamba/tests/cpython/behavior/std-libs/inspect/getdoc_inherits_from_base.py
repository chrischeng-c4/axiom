# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "getdoc_inherits_from_base"
# subject = "inspect.getdoc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getdoc: getdoc() returns __doc__, inheriting a method's docstring from a base class when the override has none"""
import inspect

class Base:
    """Base docstring."""

    def speak(self):
        """How to speak."""

class Child(Base):
    def speak(self):  # no own docstring -> inherits Base.speak.__doc__
        pass

assert inspect.getdoc(Base) == "Base docstring.", "own docstring"
assert inspect.getdoc(Child.speak) == "How to speak.", "inherited method doc"

print("getdoc_inherits_from_base OK")
