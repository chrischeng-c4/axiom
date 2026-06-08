# Regression: tuple-unpacking always set a pending ValueError because
# the length check compared a NaN-boxed `mb_list_len` result against a
# raw int literal. `c8363a68` made uncaught exceptions surface at
# module exit, so `a, b = (1, 2)` would succeed functionally but then
# exit 1 with `ValueError: unpack count mismatch`.

a, b = (1, 2)
print(a, b)

a, b, c = (10, 20, 30)
print(a, b, c)

# Nested + tuple-return
def pair():
    return (1, 2)

x, y = pair()
print(x, y)

# Star-unpack
first, *rest, last = (1, 2, 3, 4, 5)
print(first, rest, last)

# Mismatch still raises (the check itself must stay functional)
try:
    a, b = (1, 2, 3)
    print("should not reach")
except ValueError as e:
    print("caught too many")

try:
    a, b, c = (1, 2)
    print("should not reach")
except ValueError as e:
    print("caught too few")
