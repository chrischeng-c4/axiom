# Operational AssertionPass seed for the unary-operator surface.
# Surface: arithmetic unary `-` and `+` on int/float/zero literals,
# double-negation `- -x` and the cross-form `+-x` / `-+x` rewrites;
# bitwise NOT `~` on positive, negative, zero, and double-`~~`
# (involution); logical `not` over True/False, falsy int/str/list/
# None, and truthy variants; unary applied through bound variables
# (and `abs()` companion); unary embedded in arithmetic expressions
# (mixed-sign `+/-/*/// /%` results) and comparisons (sign-bearing
# `<` / `==`); unary inside list/tuple literals and conditional
# expressions. Companion to lang_arithmetic_edge and lang_bitwise_ops.
_ledger: list[int] = []

# Arithmetic unary on literals — int, float, zero
assert -5 == -5; _ledger.append(1)
assert +5 == 5; _ledger.append(1)
assert -(-5) == 5; _ledger.append(1)
assert -3.14 == -3.14; _ledger.append(1)
assert +0 == 0; _ledger.append(1)
assert -0 == 0; _ledger.append(1)

# Bitwise NOT (~): canonical identities including involution
assert ~0 == -1; _ledger.append(1)
assert ~-1 == 0; _ledger.append(1)
assert ~5 == -6; _ledger.append(1)
assert ~~5 == 5; _ledger.append(1)

# Logical `not` — True/False operands
assert (not True) == False; _ledger.append(1)
assert (not False) == True; _ledger.append(1)

# Logical `not` — falsy operands (canonical falsy set)
assert (not 0) == True; _ledger.append(1)
assert (not 1) == False; _ledger.append(1)
assert (not "") == True; _ledger.append(1)
assert (not "x") == False; _ledger.append(1)
assert (not []) == True; _ledger.append(1)
assert (not [1]) == False; _ledger.append(1)
assert (not None) == True; _ledger.append(1)

# Unary through bound variables
x = 10
assert -x == -10; _ledger.append(1)
assert +x == 10; _ledger.append(1)
assert ~x == -11; _ledger.append(1)
assert (not bool(x)) == False; _ledger.append(1)

# Unary embedded in arithmetic — mixed-sign +/-/*///
assert 5 + -3 == 2; _ledger.append(1)
assert 5 - -3 == 8; _ledger.append(1)
assert -5 * -3 == 15; _ledger.append(1)
assert 10 / -2 == -5.0; _ledger.append(1)
assert -10 // 3 == -4; _ledger.append(1)
assert -10 % 3 == 2; _ledger.append(1)

# Double-sign rewrites — `- -y`, `+-y`, `-+y`
y = 7
assert - -y == 7; _ledger.append(1)
assert +-y == -7; _ledger.append(1)
assert -+y == -7; _ledger.append(1)

# Sign-bearing comparisons
assert -5 < 0; _ledger.append(1)
assert +5 > 0; _ledger.append(1)
assert -5 < -3; _ledger.append(1)
assert -10 == -10; _ledger.append(1)
assert not (5 > 10); _ledger.append(1)

# Unary inside list / tuple literals
assert [-1, -2, -3] == [-1, -2, -3]; _ledger.append(1)
assert (-1, -2) == (-1, -2); _ledger.append(1)

# Unary + abs() companion
assert abs(-7) == 7; _ledger.append(1)
assert abs(7) == 7; _ledger.append(1)
assert abs(-3.14) == 3.14; _ledger.append(1)
assert abs(0) == 0; _ledger.append(1)

# Unary in conditional expression
assert ("neg" if -5 < 0 else "pos") == "neg"; _ledger.append(1)
assert ("neg" if 5 < 0 else "pos") == "pos"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_unary_ops {sum(_ledger)} asserts")
