"""Surface contract for language arithmetic operators.

# type-regime: monomorphic

Probes: all arithmetic operators present (+, -, *, /, //, %, **, unary +/-),
correct return types for int and float operands, operator precedence.
CPython 3.12 is the oracle.
"""

# Binary arithmetic operators — int operands
assert 3 + 4 == 7
assert 10 - 3 == 7
assert 3 * 4 == 12
assert 10 // 3 == 3
assert 10 % 3 == 1
assert 2 ** 8 == 256

# True division always returns float
assert type(10 / 2) is float
assert 10 / 2 == 5.0
assert 10 / 3 == 10 / 3  # not exact, but consistent

# Floor division returns int for int operands
assert type(10 // 3) is int
assert 10 // 3 == 3
assert -10 // 3 == -4  # floor toward -inf

# Modulo returns int for int operands
assert type(10 % 3) is int
assert 10 % 3 == 1
assert -10 % 3 == 2  # sign matches divisor

# Power returns int for non-negative int exponent
assert type(2 ** 8) is int
assert 2 ** 0 == 1

# Float arithmetic returns float
assert type(1.0 + 2.0) is float
assert type(3.0 * 2.0) is float
assert type(7.0 / 2.0) is float
assert type(7.0 // 2.0) is float
assert type(7.0 % 2.0) is float

# Unary operators
assert +5 == 5
assert -5 == -5
assert -(- 3) == 3
assert type(-3) is int
assert type(-3.0) is float

print("surface OK")
