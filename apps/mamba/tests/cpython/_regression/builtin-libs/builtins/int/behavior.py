"""Behavior contract for builtins.int.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: int() with no args returns 0
assert int() == 0, f"int() = {int()!r}, expected 0"

# Rule 2: int(int) is identity-like
assert int(5) == 5, f"int(5) = {int(5)!r}"
assert int(-3) == -3, f"int(-3) = {int(-3)!r}"

# Rule 3: int(bool)
assert int(True) == 1, f"int(True) = {int(True)!r}"
assert int(False) == 0, f"int(False) = {int(False)!r}"

# Rule 4: int(float) truncates toward zero
assert int(3.9) == 3, f"int(3.9) = {int(3.9)!r}"
assert int(-3.9) == -3, f"int(-3.9) = {int(-3.9)!r}"
assert int(0.1) == 0, f"int(0.1) = {int(0.1)!r}"

# Rule 5: int(str) parses decimal
assert int("42") == 42, f"int('42') = {int('42')!r}"
assert int("-7") == -7, f"int('-7') = {int('-7')!r}"
assert int(" 3 ") == 3, f"int(' 3 ') = {int(' 3 ')!r}"

# Rule 6: int(str, base)
assert int("ff", 16) == 255, f"int('ff', 16) = {int('ff', 16)!r}"
assert int("0xff", 16) == 255, f"int('0xff', 16) = {int('0xff', 16)!r}"
assert int("10", 2) == 2, f"int('10', 2) = {int('10', 2)!r}"
assert int("10", 8) == 8, f"int('10', 8) = {int('10', 8)!r}"
assert int("z", 36) == 35, f"int('z', 36) = {int('z', 36)!r}"

# Rule 7: int("", ...) raises ValueError
_raised = False
try:
    int("")
except ValueError:
    _raised = True
assert _raised, "int('') did not raise ValueError"

# Rule 8: int("abc") raises ValueError (non-numeric)
_raised = False
try:
    int("abc")
except ValueError:
    _raised = True
assert _raised, "int('abc') did not raise ValueError"

# Rule 9: int(None) raises TypeError
_raised = False
try:
    int(None)
except TypeError:
    _raised = True
assert _raised, "int(None) did not raise TypeError"

# Rule 10: arithmetic returns int
assert type(3 + 4) is int, f"type(3+4) = {type(3+4).__name__!r}"
assert type(10 - 7) is int, f"type(10-7) = {type(10-7).__name__!r}"
assert type(3 * 4) is int, f"type(3*4) = {type(3*4).__name__!r}"
assert type(10 // 3) is int, f"type(10//3) = {type(10//3).__name__!r}"
assert type(10 % 3) is int, f"type(10%3) = {type(10%3).__name__!r}"

# Rule 11: bitwise operations
assert 6 & 3 == 2, f"6 & 3 = {6 & 3!r}"
assert 6 | 3 == 7, f"6 | 3 = {6 | 3!r}"
assert 6 ^ 3 == 5, f"6 ^ 3 = {6 ^ 3!r}"
assert ~0 == -1, f"~0 = {~0!r}"
assert 1 << 3 == 8, f"1 << 3 = {1 << 3!r}"
assert 16 >> 2 == 4, f"16 >> 2 = {16 >> 2!r}"

# Rule 12: int.bit_length()
assert (0).bit_length() == 0, f"(0).bit_length() = {(0).bit_length()!r}"
assert (1).bit_length() == 1, f"(1).bit_length() = {(1).bit_length()!r}"
assert (255).bit_length() == 8, f"(255).bit_length() = {(255).bit_length()!r}"

# Rule 13: int.__index__ — int is its own index
assert (7).__index__() == 7, f"(7).__index__() = {(7).__index__()!r}"

# Rule 14: divmod with int
q, r = divmod(17, 5)
assert q == 3, f"divmod(17,5)[0] = {q!r}"
assert r == 2, f"divmod(17,5)[1] = {r!r}"

# Rule 15: pow(base, exp, mod)
assert pow(2, 10, 1000) == 24, f"pow(2,10,1000) = {pow(2,10,1000)!r}"

print("behavior OK")
