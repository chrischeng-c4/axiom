# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""int() constructor protocol: no-args, keyword forms, the __int__ /
__index__ conversion hooks, and very large results.
"""

import sys

# No arguments yields zero.
assert int() == 0
print("no_args: ok")

# The value may be passed positionally, the base by keyword.
assert int("100", base=2) == 4
print("keyword_base: ok")

# `x` is not an accepted keyword name; using it is a TypeError.
for call in (lambda: int(x=1.2), lambda: int(x="100", base=2)):
    try:
        call()
        print("keyword_x: no_raise")
        break
    except TypeError:
        pass
else:
    print("keyword_x: TypeError")

# A base with no value string is also a TypeError.
for base in (10, 0):
    try:
        int(base=base)
        print("base_only: no_raise")
        break
    except TypeError:
        pass
else:
    print("base_only: TypeError")

# __int__ on a subclass drives the conversion result.
class WithInt(int):
    def __int__(self):
        return 42


assert WithInt(7) == 7          # the stored int value
assert int(WithInt(7)) == 42    # __int__ wins for int()
print("dunder_int: ok")

# __int__ must return a real int; returning a float is a TypeError.
class BadInt(int):
    def __int__(self):
        return 42.0


try:
    int(BadInt(7))
    print("bad_dunder_int: no_raise")
except TypeError:
    print("bad_dunder_int: TypeError")

# A subclass that only defines __index__ keeps its own int value.
class WithIndex(int):
    def __index__(self):
        return 99


assert int(WithIndex(7)) == 7
print("dunder_index: ok")

# Float-derived ints and 600-digit ints are still ints; >> matches // 2.
assert isinstance(int(1e100), int)
assert isinstance(int(-1e100), int)
assert isinstance(int("1" * 600), int)
big_negative = -1 - sys.maxsize
assert big_negative >> 1 == big_negative // 2
print("big_ints: ok")

print("int_constructor OK")
