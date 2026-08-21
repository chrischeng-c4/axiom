# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "methodcaller_args_kwargs_defaults"
# subject = "operator.methodcaller"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.methodcaller: methodcaller binds positional and keyword arguments at construction and forwards them to the named method, while the method's own defaults still apply"""
import operator

class Target:
    def add_first_two(self, *args, **kwds):
        return args[0] + args[1]

    def with_default(self, f=42):
        return f

    def kwargs_only(*args, **kwds):
        return (kwds["name"], kwds["self"])


t = Target()
assert operator.methodcaller("add_first_two", 1, 2)(t) == 3, "bound positional args"
assert operator.methodcaller("with_default")(t) == 42, "method default arg"
assert operator.methodcaller("with_default", f=5)(t) == 5, "bound kwarg"
assert operator.methodcaller("kwargs_only", name="spam", self="eggs")(t) == (
    "spam", "eggs"
), "kwargs forwarded"
assert operator.methodcaller("upper")("hello") == "HELLO", "upper on str"

print("methodcaller_args_kwargs_defaults OK")
