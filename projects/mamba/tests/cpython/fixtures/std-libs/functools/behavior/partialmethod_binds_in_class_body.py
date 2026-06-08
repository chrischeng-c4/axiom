# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partialmethod_binds_in_class_body"
# subject = "functools.partialmethod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partialmethod: partialmethod binds a positional-only argument when declared in a class body"""
import functools


# partialmethod binds a leading positional-only argument inside the class
# body, so add_one(2) calls add(self, 1, 2).
class Adder:
    def add(self, a, b, /):
        return a + b

    add_one = functools.partialmethod(add, 1)


assert Adder().add_one(2) == 3, "partialmethod positional-only"
assert Adder().add_one(40) == 41, "partialmethod reused"


# partialmethod(None, ...) with a non-callable first arg raises TypeError
# at class definition time.
try:

    class Bad:
        m = functools.partialmethod(None, 1)

    raise AssertionError("expected TypeError for partialmethod(None, ...)")
except TypeError:
    pass

print("partialmethod_binds_in_class_body OK")
