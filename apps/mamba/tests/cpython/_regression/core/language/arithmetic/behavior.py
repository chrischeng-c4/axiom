"""Behavior contract for language arithmetic operators.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: integer arithmetic
assert 5 + 3 == 8
assert 10 - 7 == 3
assert 6 * 7 == 42
assert 2 ** 10 == 1024
assert 17 // 5 == 3
assert 17 % 5 == 2

# Rule 2: floor division semantics (toward -inf)
assert 7 // 2 == 3
assert -7 // 2 == -4   # floor, not truncate
assert 7 // -2 == -4
assert -7 // -2 == 3

# Rule 3: modulo sign matches divisor
assert 7 % 3 == 1
assert -7 % 3 == 2    # positive because divisor is positive
assert 7 % -3 == -2   # negative because divisor is negative
assert -7 % -3 == -1

# Rule 4: invariant a == (a // b) * b + (a % b)
for a, b in [(17, 5), (-17, 5), (17, -5), (-17, -5), (0, 7), (7, 1)]:
    assert a == (a // b) * b + (a % b), f"invariant failed: {a}, {b}"

# Rule 5: true division
assert 10 / 4 == 2.5
assert 1 / 3 != 0         # not integer division
assert type(1 / 3) is float

# Rule 6: mixed int + float → float
assert type(1 + 1.0) is float
assert type(1 * 2.0) is float
assert 1 + 1.0 == 2.0

# Rule 7: operator precedence (standard math rules)
assert 2 + 3 * 4 == 14     # * before +
assert (2 + 3) * 4 == 20   # parentheses override
assert 2 ** 3 ** 2 == 512  # ** right-associative: 2**(3**2) = 2**9 = 512
assert 8 / 2 * 4 == 16.0   # / and * left-to-right

# Rule 8: augmented assignment
x = 10
x += 5;  assert x == 15
x -= 3;  assert x == 12
x *= 2;  assert x == 24
x //= 5; assert x == 4
x %= 3;  assert x == 1
x **= 4; assert x == 1
x = 10
x /= 4;  assert x == 2.5

# Rule 9: ZeroDivisionError
for op in [lambda: 1 // 0, lambda: 1 % 0, lambda: 1 / 0]:
    _raised = False
    try:
        op()
    except ZeroDivisionError:
        _raised = True
    assert _raised, f"{op} did not raise ZeroDivisionError"

# Rule 10: unary negation distributes
assert -(3 + 4) == -7
assert -(-5) == 5
assert -(3 * 4) == -12

print("behavior OK")
