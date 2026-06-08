# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "getsizeof_default_fallback"
# subject = "sys.getsizeof"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.getsizeof: the second positional arg is a default size: ignored for a normal object (real size returned), but returned when the object's __sizeof__ raises"""
import sys


class _NoSize:
    def __sizeof__(self):
        raise TypeError("boom")


# A normal object reports its real size; the default arg is ignored.
_real = sys.getsizeof(object(), 1234)
assert isinstance(_real, int) and _real > 0 and _real != 1234, \
    f"normal object size = {_real!r}"
# When __sizeof__ raises, getsizeof returns the supplied default instead.
assert sys.getsizeof(_NoSize(), 1234) == 1234, "default returned on __sizeof__ failure"
print("getsizeof_default_fallback OK")
