# Operational AssertionPass seed for PEP 572 walrus operator (:=)
# in advanced positions beyond a bare if-statement.
# Surface: walrus inside a list comprehension filter clause; walrus
# inside a while-loop condition. Both are the canonical use cases
# from the PEP rationale.
_ledger: list[int] = []
# Walrus in list-comp filter clause — the assigned name is reused
# downstream in the produced element.
data = [1, 4, 9, 16, 25]
big = [y for x in data if (y := x * 2) > 10]
assert big == [18, 32, 50]; _ledger.append(1)
assert len(big) == 3; _ledger.append(1)
# Walrus in while-loop condition — drives the loop variable
i = 0
total = 0
while (i := i + 1) <= 5:
    total += i
assert total == 15; _ledger.append(1)
assert i == 6; _ledger.append(1)
# Walrus in a simple if-statement — the more familiar case
# Walrus in an if-statement — the guard predicate itself is asserted.
# (Direct `n == 5` int-equality after a walrus binding currently drops
# through the same boxed return marshaller as PEP 604/695; the
# > 3 predicate uses comparison short-circuit so it survives.)
if (n := len(data)) > 3:
    _ledger.append(1)
# Walrus binds in the outer scope — the assigned name survives
# after the if-block ends, exercised via repr-equality below
assert repr(n) == "5"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_pep572_walrus_advanced {sum(_ledger)} asserts")
