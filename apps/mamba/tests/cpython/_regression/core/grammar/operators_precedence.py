# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/grammar: operator precedence and expression grammar (CPython 3.12 oracle).

Distilled from CPython GrammarTests for unary/binary/bitwise ops, chained
comparison, parenthesized evaluation, and the conditional (ternary) operator.
"""

# Unary operators and their interaction with arithmetic precedence.
assert +1 == 1
assert -1 == -1
assert ~1 == -2
assert ---1 == -1
assert -2 ** 2 == -4          # ** binds tighter than unary minus
assert (-2) ** 2 == 4
print("unary: ok")

# Bitwise precedence: ~ then & then ^ then |.
assert ~1 ^ 1 & 1 | 1 & 1 ^ -1 == ((~1) ^ ((1 & 1)) | ((1 & 1) ^ (-1)))
assert 1 | 2 & 3 == 3          # & before |
assert 1 ^ 2 | 4 == 7
print("bitwise: ok")

# Floor division is left-associative; parens override.
assert 16 // 4 // 2 == 2
assert 16 // (4 // 2) == 8
print("floordiv_assoc: ok")

# Chained comparison evaluates pairwise with `and`, not as one big compare.
assert (1 < 2 < 3) is True
assert (1 < 2 > 1) is True
assert (1 < 2 < 1) is False
assert (3 > 2 > 1 == 1) is True
print("chained_compare: ok")

# `is` / `is not` / `in` / `not in` chain too.
x = []
assert (x is x) is True
assert (x is not x) is False
assert (1 in (1, 2)) is True
assert (3 not in (1, 2)) is True
print("identity_membership: ok")

# Parenthesized boolean evaluation binds as written.
a, b = 2, 3
assert (False is (a is b)) is True       # a is b -> False, False is False -> True
assert (not (False is a) is b) is True
print("paren_eval: ok")

# Conditional expression precedence vs and/or/not/arithmetic.
assert (5 if 1 else 0) == 5
assert (5 and 6 if 0 else 1) == 1
assert (5 and (6 if 1 else 1)) == 6
assert (not 5 if 1 else 1) is False
assert (6 + 1 if 1 else 2) == 7
assert (6 * 2 if 1 else 4) == 12
print("ternary: ok")

# `or` short-circuits to the first truthy operand (a tuple here).
i, j = (1, -1) or (-1, 1)
assert i == 1 and j == -1
print("operators_precedence OK")
