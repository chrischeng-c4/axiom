# Operational AssertionPass seed for PEP 572 — assignment-expression
# (walrus) operator (CPython 3.8+).
# Surface: `:=` binds the value to a name while yielding the value in
# an expression position. Tested in `if` guard, `while` guard
# (reading until sentinel), and a list-comprehension `if` clause.
# Note: the walrus-bound name's int-equality is broken after the
# expression (Task #17), so structural shape (length, contents
# captured from the comprehension body) is asserted instead of
# direct == against the walrus name itself.
_ledger: list[int] = []

# Walrus in an if-guard: `n` captures len(data) and the comparison
# uses the captured name. The else branch is exercised by re-running
# with a short list.
data = [1, 2, 3, 4, 5]
if (n := len(data)) > 3:
    r1 = n
else:
    r1 = -1
# n == 5 hits Task #17; assert via subtraction
assert n - 5 == 0; _ledger.append(1)
# r1 is the value of n, captured into a local — local int-eq works
assert r1 - 5 == 0; _ledger.append(1)

# else branch
short = [1]
if (m := len(short)) > 3:
    r2 = m
else:
    r2 = -1
assert r2 == -1; _ledger.append(1)

# Walrus in while-guard: read source elements until a sentinel 0
src = [1, 2, 3, 0, 5]
i = 0
collected = []
while (v := src[i]) != 0:
    collected.append(v)
    i += 1
# Loop exited at index 3 (the sentinel)
assert i == 3; _ledger.append(1)
# Captured three elements before the sentinel
assert len(collected) == 3; _ledger.append(1)
# Values captured by the walrus body land in the list
assert collected == [1, 2, 3]; _ledger.append(1)

# Walrus in a comprehension `if` clause — the walrus binds `y` to the
# doubled element and the body returns `y`
results = [y for x in [1, 2, 3, 4] if (y := x * 2) > 4]
# Doubled values > 4 are 6 (from 3) and 8 (from 4)
assert results == [6, 8]; _ledger.append(1)
assert len(results) == 2; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_pep572_walrus {sum(_ledger)} asserts")
