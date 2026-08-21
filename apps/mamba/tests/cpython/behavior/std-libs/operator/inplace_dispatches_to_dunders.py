# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "inplace_dispatches_to_dunders"
# subject = "operator.iadd"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.iadd: each in-place function dispatches to its __i*__ dunder; iconcat falls back to __iadd__ when no __iconcat__ exists; in-place numeric ops on built-in ints behave like the binary form"""
import operator

class Recorder:
    """Each in-place dunder returns its own name so dispatch is observable."""

    def __iadd__(self, other):
        return "iadd"

    def __iand__(self, other):
        return "iand"

    def __ifloordiv__(self, other):
        return "ifloordiv"

    def __ilshift__(self, other):
        return "ilshift"

    def __imod__(self, other):
        return "imod"

    def __imul__(self, other):
        return "imul"

    def __imatmul__(self, other):
        return "imatmul"

    def __ior__(self, other):
        return "ior"

    def __ipow__(self, other):
        return "ipow"

    def __irshift__(self, other):
        return "irshift"

    def __isub__(self, other):
        return "isub"

    def __itruediv__(self, other):
        return "itruediv"

    def __ixor__(self, other):
        return "ixor"

    def __getitem__(self, key):
        return 0


r = Recorder()
cases = [
    (operator.iadd, "iadd"),
    (operator.iand, "iand"),
    (operator.ifloordiv, "ifloordiv"),
    (operator.ilshift, "ilshift"),
    (operator.imod, "imod"),
    (operator.imul, "imul"),
    (operator.imatmul, "imatmul"),
    (operator.ior, "ior"),
    (operator.ipow, "ipow"),
    (operator.irshift, "irshift"),
    (operator.isub, "isub"),
    (operator.itruediv, "itruediv"),
    (operator.ixor, "ixor"),
]
for func, expected in cases:
    result = func(r, 5)
    assert result == expected, f"{func.__name__} -> {result!r}"

# iconcat falls back to __iadd__ when no __iconcat__ exists.
assert operator.iconcat(r, r) == "iadd", "iconcat falls back to iadd"

# In-place numeric ops on built-in ints behave like the binary form.
assert operator.iadd(3, 4) == 7, "iadd on int"
assert operator.imul(6, 7) == 42, "imul on int"

print("inplace_dispatches_to_dunders OK")
