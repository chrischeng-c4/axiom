# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Sequence unpacking — #2811.
#
# Covers tuple/list/iterable unpacking, nested unpacking, and starred
# assignment (PEP 3132). Includes ValueError cases where the rhs has
# the wrong arity for a non-starred lhs. Each clause prints with the
# `[unpacking]` tag so failure output names the semantic area.

# 1. Plain tuple unpacking.
a, b = (1, 2)
print("tuple a,b=", a, b, "[unpacking]")

# 2. Plain list unpacking.
c, d, e = [10, 20, 30]
print("list c,d,e=", c, d, e, "[unpacking]")

# 3. Unpacking from an arbitrary iterable.
def gen():
    yield 7
    yield 8
    yield 9
g1, g2, g3 = gen()
print("gen g1,g2,g3=", g1, g2, g3, "[unpacking: iterable]")

# 4. Nested unpacking.
(x1, (x2, x3), x4) = (1, (2, 3), 4)
print("nested x1..x4=", x1, x2, x3, x4, "[unpacking: nested]")

# 5. Starred at the end.
first, *rest = [1, 2, 3, 4]
print("first,*rest=", first, rest, "[unpacking: starred-tail]")

# 6. Starred at the start.
*head, last = [1, 2, 3, 4]
print("*head,last=", head, last, "[unpacking: starred-head]")

# 7. Starred in the middle.
lo, *mid, hi = [1, 2, 3, 4, 5]
print("lo,*mid,hi=", lo, mid, hi, "[unpacking: starred-mid]")

# 8. Starred catches empty when the iterable is exactly the fixed part.
only_first, *empty = [42]
print("only_first,*empty=", only_first, empty, "[unpacking: starred-empty]")

# 9. ValueError: too few values to unpack.
raised_few = False
try:
    p, q, r = [1, 2]
except ValueError:
    raised_few = True
print("too-few ValueError=", raised_few, "[unpacking: arity]")

# 10. ValueError: too many values to unpack.
raised_many = False
try:
    s, t = [1, 2, 3]
except ValueError:
    raised_many = True
print("too-many ValueError=", raised_many, "[unpacking: arity]")
