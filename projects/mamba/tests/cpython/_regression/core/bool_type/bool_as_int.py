# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# bool ≤ int (#1680). Python defines `bool` as a subclass of `int`:
# `isinstance(True, int) is True`. Anywhere int is expected — range
# bounds, slice components, list indices, function-argument positions —
# bool must be accepted and behave as 0/1.

# range with bool args.
print(list(range(True)))
print(list(range(False, True)))
print(list(range(0, 5, True)))

# Indexing.
xs = [10, 20, 30, 40]
print(xs[True])
print(xs[False])

# Slicing with bool bounds and step.
print(xs[False:True])
print(xs[True:])
print(xs[::True])

# Function call: int param, bool argument.
def f(n: int) -> int:
    return n + 1
print(f(True))
print(f(False))
