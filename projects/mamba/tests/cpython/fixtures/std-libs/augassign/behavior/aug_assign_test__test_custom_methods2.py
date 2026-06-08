# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "augassign"
# dimension = "behavior"
# case = "aug_assign_test__test_custom_methods2"
# subject = "cpython.test_augassign.AugAssignTest.testCustomMethods2"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_augassign.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_augassign.py::AugAssignTest::testCustomMethods2
"""Auto-ported test: AugAssignTest::testCustomMethods2 (CPython 3.12 oracle)."""


output = []


class testall:
    def __add__(self, val):
        output.append("__add__ called")

    def __radd__(self, val):
        output.append("__radd__ called")

    def __iadd__(self, val):
        output.append("__iadd__ called")
        return self

    def __sub__(self, val):
        output.append("__sub__ called")

    def __rsub__(self, val):
        output.append("__rsub__ called")

    def __isub__(self, val):
        output.append("__isub__ called")
        return self

    def __mul__(self, val):
        output.append("__mul__ called")

    def __rmul__(self, val):
        output.append("__rmul__ called")

    def __imul__(self, val):
        output.append("__imul__ called")
        return self

    def __matmul__(self, val):
        output.append("__matmul__ called")

    def __rmatmul__(self, val):
        output.append("__rmatmul__ called")

    def __imatmul__(self, val):
        output.append("__imatmul__ called")
        return self

    def __floordiv__(self, val):
        output.append("__floordiv__ called")
        return self

    def __ifloordiv__(self, val):
        output.append("__ifloordiv__ called")
        return self

    def __rfloordiv__(self, val):
        output.append("__rfloordiv__ called")
        return self

    def __truediv__(self, val):
        output.append("__truediv__ called")
        return self

    def __rtruediv__(self, val):
        output.append("__rtruediv__ called")
        return self

    def __itruediv__(self, val):
        output.append("__itruediv__ called")
        return self

    def __mod__(self, val):
        output.append("__mod__ called")

    def __rmod__(self, val):
        output.append("__rmod__ called")

    def __imod__(self, val):
        output.append("__imod__ called")
        return self

    def __pow__(self, val):
        output.append("__pow__ called")

    def __rpow__(self, val):
        output.append("__rpow__ called")

    def __ipow__(self, val):
        output.append("__ipow__ called")
        return self

    def __or__(self, val):
        output.append("__or__ called")

    def __ror__(self, val):
        output.append("__ror__ called")

    def __ior__(self, val):
        output.append("__ior__ called")
        return self

    def __and__(self, val):
        output.append("__and__ called")

    def __rand__(self, val):
        output.append("__rand__ called")

    def __iand__(self, val):
        output.append("__iand__ called")
        return self

    def __xor__(self, val):
        output.append("__xor__ called")

    def __rxor__(self, val):
        output.append("__rxor__ called")

    def __ixor__(self, val):
        output.append("__ixor__ called")
        return self

    def __rshift__(self, val):
        output.append("__rshift__ called")

    def __rrshift__(self, val):
        output.append("__rrshift__ called")

    def __irshift__(self, val):
        output.append("__irshift__ called")
        return self

    def __lshift__(self, val):
        output.append("__lshift__ called")

    def __rlshift__(self, val):
        output.append("__rlshift__ called")

    def __ilshift__(self, val):
        output.append("__ilshift__ called")
        return self


x = testall()
x + 1
1 + x
x += 1

x - 1
1 - x
x -= 1

x * 1
1 * x
x *= 1

x @ 1
1 @ x
x @= 1

x / 1
1 / x
x /= 1

x // 1
1 // x
x //= 1

x % 1
1 % x
x %= 1

x ** 1
1 ** x
x **= 1

x | 1
1 | x
x |= 1

x & 1
1 & x
x &= 1

x ^ 1
1 ^ x
x ^= 1

x >> 1
1 >> x
x >>= 1

x << 1
1 << x
x <<= 1

assert output == """\
__add__ called
__radd__ called
__iadd__ called
__sub__ called
__rsub__ called
__isub__ called
__mul__ called
__rmul__ called
__imul__ called
__matmul__ called
__rmatmul__ called
__imatmul__ called
__truediv__ called
__rtruediv__ called
__itruediv__ called
__floordiv__ called
__rfloordiv__ called
__ifloordiv__ called
__mod__ called
__rmod__ called
__imod__ called
__pow__ called
__rpow__ called
__ipow__ called
__or__ called
__ror__ called
__ior__ called
__and__ called
__rand__ called
__iand__ called
__xor__ called
__rxor__ called
__ixor__ called
__rshift__ called
__rrshift__ called
__irshift__ called
__lshift__ called
__rlshift__ called
__ilshift__ called
""".splitlines()

print("AugAssignTest::testCustomMethods2: ok")
