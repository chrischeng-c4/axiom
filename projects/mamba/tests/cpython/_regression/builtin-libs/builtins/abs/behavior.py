"""Behavior contract for builtins.abs.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: abs of positive int
result = abs(5)
assert result == 5, f"abs(5) = {result!r}, expected 5"

# Rule 2: abs of negative int
result = abs(-5)
assert result == 5, f"abs(-5) = {result!r}, expected 5"

# Rule 3: abs of zero
result = abs(0)
assert result == 0, f"abs(0) = {result!r}, expected 0"

# Rule 4: abs of positive float
result = abs(3.14)
assert result == 3.14, f"abs(3.14) = {result!r}, expected 3.14"

# Rule 5: abs of negative float
result = abs(-3.14)
assert result == 3.14, f"abs(-3.14) = {result!r}, expected 3.14"

# Rule 6: abs returns int for int input
result = abs(-7)
assert type(result) is int, \
    f"type(abs(-7)) = {type(result).__name__!r}, expected 'int'"

# Rule 7: abs returns float for float input
result = abs(-1.5)
assert type(result) is float, \
    f"type(abs(-1.5)) = {type(result).__name__!r}, expected 'float'"

# Rule 8: abs of complex — returns float magnitude
result = abs(3 + 4j)
assert result == 5.0, f"abs(3+4j) = {result!r}, expected 5.0"

# Rule 9: abs of complex returns float
result = abs(-3 - 4j)
assert type(result) is float, \
    f"type(abs(-3-4j)) = {type(result).__name__!r}, expected 'float'"

# Rule 10: large negative int
result = abs(-1_000_000_000)
assert result == 1_000_000_000, \
    f"abs(-1_000_000_000) = {result!r}, expected 1000000000"

# Rule 11: custom __abs__ is respected
class _V:
    def __abs__(self):
        return 42

result = abs(_V())
assert result == 42, f"abs(custom __abs__) = {result!r}, expected 42"

# Rule 12: TypeError for str (no __abs__)
_raised = False
try:
    abs("x")
except TypeError:
    _raised = True
assert _raised, "abs('x') did not raise TypeError"

# Rule 13: abs(-0.0) == 0.0
result = abs(-0.0)
assert result == 0.0, f"abs(-0.0) = {result!r}, expected 0.0"
assert type(result) is float, \
    f"type(abs(-0.0)) = {type(result).__name__!r}, expected 'float'"

print("behavior OK")
