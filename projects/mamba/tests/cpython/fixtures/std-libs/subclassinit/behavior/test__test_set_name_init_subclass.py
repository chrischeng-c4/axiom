# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_set_name_init_subclass"
# subject = "cpython.test_subclassinit.Test.test_set_name_init_subclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_set_name_init_subclass
"""Auto-ported test: Test::test_set_name_init_subclass (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
class Descriptor:

    def __set_name__(self, owner, name):
        self.owner = owner
        self.name = name

class Meta(type):

    def __new__(cls, name, bases, ns):
        self = super().__new__(cls, name, bases, ns)
        self.meta_owner = self.owner
        self.meta_name = self.name
        return self

class A:

    def __init_subclass__(cls):
        cls.owner = cls.d.owner
        cls.name = cls.d.name

class B(A, metaclass=Meta):
    d = Descriptor()

assert B.owner is B

assert B.name == 'd'

assert B.meta_owner is B

assert B.name == 'd'
print("Test::test_set_name_init_subclass: ok")
