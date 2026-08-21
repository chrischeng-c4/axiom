# mamba-xfail: unary `-` on user-defined classes is rejected at type-check
# ("unary `-` requires numeric type") instead of dispatching to __neg__.
# Arithmetic/reflected/comparison clauses (1-4) already pass on mamba; the
# unary __neg__ clause (5) is the runtime gap that gates the xfail.
#
# Operator dunder dispatch — #2786.
#
# Covers user-defined operator dunder dispatch for arithmetic, reflected, and
# unary/comparison ops, including mixed primitive/user operands.
#
# Clauses:
#   1. user + user            -> __add__ on left operand.
#   2. user + primitive       -> __add__ (left handles the int).
#   3. primitive + user       -> int.__add__ returns NotImplemented, so
#                                Python falls back to __radd__ on the user.
#   4. user < user / user < n -> __lt__ on left operand.
#   5. -user                  -> __neg__ on operand.
#
# The fixture fails (CPython reference mismatch) if the runtime bypasses any
# of these user dunders or returns the wrong operand. Every print line is
# tagged with `[operator-dispatch]` so failure output names the area.

class Box:
    def __init__(self, v):
        self.v = v

    def __add__(self, other):
        if isinstance(other, Box):
            return Box(self.v + other.v)
        return Box(self.v + other)

    def __radd__(self, other):
        # Only invoked when the left operand (e.g. an int) returned
        # NotImplemented for `other + self`.
        return Box(other + self.v)

    def __lt__(self, other):
        if isinstance(other, Box):
            return self.v < other.v
        return self.v < other

    def __neg__(self):
        return Box(-self.v)

    def __repr__(self):
        return "Box(" + str(self.v) + ")"


a = Box(3)
b = Box(4)

# 1. user + user -> __add__
c = a + b
print("a+b=", c, "[operator-dispatch]")

# 2. user + primitive -> __add__
d = a + 10
print("a+10=", d, "[operator-dispatch]")

# 3. primitive + user -> __radd__ (reflected)
e = 100 + a
print("100+a=", e, "[operator-dispatch: __radd__]")

# 4. comparison __lt__
print("a<b=", a < b, "[operator-dispatch]")
print("b<a=", b < a, "[operator-dispatch]")
print("a<5=", a < 5, "[operator-dispatch]")

# 5. unary __neg__
print("-a=", -a, "[operator-dispatch: __neg__]")
print("--a.v==a.v=", (-(-a)).v == a.v, "[operator-dispatch]")
