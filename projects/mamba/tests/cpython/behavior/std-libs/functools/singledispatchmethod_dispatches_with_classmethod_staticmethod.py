# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "singledispatchmethod_dispatches_with_classmethod_staticmethod"
# subject = "functools.singledispatchmethod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.singledispatchmethod: singledispatchmethod dispatches a method by its first non-self argument and stacks with classmethod/staticmethod, threading cls/self correctly"""
import functools


# Plain method dispatch: each registered type sets a different value, and
# only the receiving instance is mutated.
class Recorder:
    @functools.singledispatchmethod
    def handle(self, arg):
        self.arg = "base"

    @handle.register(int)
    def _(self, arg):
        self.arg = "int"

    @handle.register(str)
    def _(self, arg):
        self.arg = "str"


a = Recorder()
a.handle(0)
assert a.arg == "int", "int -> int"

fresh = Recorder()
assert not hasattr(fresh, "arg"), "untouched instance has no arg"

a.handle("x")
assert a.arg == "str", "str -> str"

a.handle(0.0)
assert a.arg == "base", "float -> base"


# classmethod stacking: dispatch works and cls is threaded through.
class Factory:
    def __init__(self, tag):
        self.tag = tag

    @functools.singledispatchmethod
    @classmethod
    def make(cls, arg):
        return cls("base")

    @make.register(int)
    @classmethod
    def _(cls, arg):
        return cls("int")

    @make.register(str)
    @classmethod
    def _(cls, arg):
        return cls("str")


assert Factory.make(0).tag == "int", "classmethod int"
assert Factory.make("").tag == "str", "classmethod str"
assert Factory.make(0.0).tag == "base", "classmethod float"


# staticmethod stacking: no self/cls is passed to the implementations.
class Checker:
    @functools.singledispatchmethod
    @staticmethod
    def check(arg):
        return "base"

    @check.register(int)
    @staticmethod
    def _(arg):
        return isinstance(arg, int)

    @check.register(str)
    @staticmethod
    def _(arg):
        return isinstance(arg, str)


assert Checker.check(0) is True, "staticmethod int"
assert Checker.check("") is True, "staticmethod str"
assert Checker.check(0.0) == "base", "staticmethod float"

print("singledispatchmethod_dispatches_with_classmethod_staticmethod OK")
