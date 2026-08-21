# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""classmethod / staticmethod __func__ wrapping (CPython 3.12 oracle).

Wrapping a plain function in classmethod() or staticmethod() preserves
the original via __func__. Bound through a class, a classmethod receives
the class as first argument and a staticmethod receives none.
"""


def plain(*args):
    return args


# The wrappers expose the original function unchanged via __func__.
assert classmethod(plain).__func__ is plain
assert staticmethod(plain).__func__ is plain


class C:
    @classmethod
    def cm(cls, x):
        return (cls, x)

    @staticmethod
    def sm(x):
        return x


# classmethod injects the class; staticmethod passes args straight through.
cls, value = C.cm(7)
assert cls is C
assert value == 7
assert C.sm(9) == 9

# Reachable identically from an instance.
inst = C()
assert inst.cm(1)[0] is C
assert inst.sm(2) == 2

print("classmethod_staticmethod_func OK")
