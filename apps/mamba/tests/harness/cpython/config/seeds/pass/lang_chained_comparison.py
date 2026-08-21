# Operational AssertionPass seed for the chained-comparison surface.
# Surface: `a < b < c` evaluates as `(a < b) and (b < c)` (each
# operand evaluated once, short-circuiting on the first failure);
# the chain supports mixing `<`, `<=`, `>`, `>=`, `==`, `!=` in any
# combination; chains generalize to length-4 (`a<b<c<d`); operands
# may be int literals, float literals, negative numbers, strings
# (lexicographic), bound variables, subscript reads, and function
# call results; the common `0 < n < 10` range-membership idiom
# evaluates to True iff n is strictly inside (0, 10); a chained
# comparison can sit on the RHS of `and`/`or`. Companion to
# lang_unary_ops (which covers single-operator comparisons).
_ledger: list[int] = []

# Strict-less three-chain
assert (1 < 2 < 3) == True; _ledger.append(1)
assert (1 < 2 < 1) == False; _ledger.append(1)

# Strict-greater three-chain
assert (3 > 2 > 1) == True; _ledger.append(1)

# Direction reversal mid-chain
assert (1 < 2 > 0) == True; _ledger.append(1)

# All-equal chain (transitive equality)
assert (1 == 1 == 1) == True; _ledger.append(1)
assert (1 == 1 == 2) == False; _ledger.append(1)

# Non-strict chains
assert (1 <= 1 <= 1) == True; _ledger.append(1)
assert (1 <= 2 <= 3) == True; _ledger.append(1)
assert (3 >= 2 >= 1) == True; _ledger.append(1)

# Inequality chain
assert (1 != 2 != 3) == True; _ledger.append(1)
assert (1 != 2 != 2) == False; _ledger.append(1)

# Bound-variable chains
a, b, c = 1, 2, 3
assert (a < b < c) == True; _ledger.append(1)
assert (a < b > 0) == True; _ledger.append(1)
assert (a == a == a) == True; _ledger.append(1)
assert (a < c and a < b) == True; _ledger.append(1)

# Length-4 chains
assert (1 < 2 < 3 < 4) == True; _ledger.append(1)
assert (1 < 2 < 3 < 2) == False; _ledger.append(1)

# Mixed comparison operators
assert (1 <= 2 < 3) == True; _ledger.append(1)
assert (3 >= 2 > 1) == True; _ledger.append(1)
assert (1 < 2 == 2) == True; _ledger.append(1)
assert (1 < 2 != 1) == True; _ledger.append(1)

# Float operands
assert (1.0 < 2.0 < 3.0) == True; _ledger.append(1)
assert (1.5 < 2.5 < 3.5) == True; _ledger.append(1)

# Negative-number chains
assert (-3 < -2 < -1) == True; _ledger.append(1)
assert (-1 > -2 > -3) == True; _ledger.append(1)

# String chains (lexicographic order)
assert ("a" < "b" < "c") == True; _ledger.append(1)
assert ("apple" < "banana" < "cherry") == True; _ledger.append(1)

# `0 < n < limit` range-membership idiom
n = 5
assert (0 < n < 10) == True; _ledger.append(1)
assert (0 < n < 3) == False; _ledger.append(1)
assert (10 < n < 20) == False; _ledger.append(1)

# Equality + strict-less mixed
assert (1 == 1 < 2) == True; _ledger.append(1)
assert (1 == 1 < 0) == False; _ledger.append(1)

# Operands from subscript reads
xs = [1, 2, 3]
assert (xs[0] < xs[1] < xs[2]) == True; _ledger.append(1)

# Operand from a function-call result
def f() -> int:
    return 5
assert (1 < f() < 10) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_chained_comparison {sum(_ledger)} asserts")
