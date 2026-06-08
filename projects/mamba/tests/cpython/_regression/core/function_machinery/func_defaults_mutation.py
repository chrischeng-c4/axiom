# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Runtime mutation of __defaults__ (CPython 3.12 oracle).

A function's default arguments live in the writable __defaults__ tuple.
Assigning it makes previously-required parameters optional; deleting it
makes them required again. Calls reflect the change immediately.
"""


def first(a, b):
    return a + b


def second(a=1, b=2):
    return a + b


# No defaults -> None; with defaults -> the tuple of values.
assert first.__defaults__ is None
assert second.__defaults__ == (1, 2)

# Installing defaults makes both params optional.
first.__defaults__ = (1, 2)
assert first.__defaults__ == (1, 2)
assert first() == 3          # both defaulted
assert first(3) == 5         # b defaulted
assert first(3, 5) == 8      # both supplied

# Deleting __defaults__ restores the required-argument contract.
del second.__defaults__
assert second.__defaults__ is None
try:
    second()
    raise AssertionError("expected TypeError after deleting __defaults__")
except TypeError:
    pass

print("func_defaults_mutation OK")
