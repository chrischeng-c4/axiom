"""Behavior contract for language control flow.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: if / elif / else selects correct branch
def _classify(n: int) -> str:
    if n > 0:
        return "pos"
    elif n < 0:
        return "neg"
    else:
        return "zero"
assert _classify(5) == "pos"
assert _classify(-3) == "neg"
assert _classify(0) == "zero"

# Rule 2: for loop iterates over range
acc = []
for i in range(4):
    acc.append(i)
assert acc == [0, 1, 2, 3], f"for range = {acc!r}"

# Rule 3: for loop over list
words = ["a", "b", "c"]
out = []
for w in words:
    out.append(w.upper())
assert out == ["A", "B", "C"], f"for list = {out!r}"

# Rule 4: while loop with condition
n = 10
total = 0
while n > 0:
    total += n
    n -= 1
assert total == 55, f"while sum = {total!r}"

# Rule 5: break exits immediately
found = -1
for i in range(100):
    if i * i > 50:
        found = i
        break
assert found == 8, f"break found = {found!r}"  # 8*8=64 > 50

# Rule 6: continue skips rest of body
odds = []
for i in range(10):
    if i % 2 == 0:
        continue
    odds.append(i)
assert odds == [1, 3, 5, 7, 9], f"continue odds = {odds!r}"

# Rule 7: for-else — else runs when no break
def _has(lst: list, v: int) -> bool:
    for x in lst:
        if x == v:
            break
    else:
        return False
    return True
assert _has([1, 2, 3], 2) == True
assert _has([1, 2, 3], 9) == False

# Rule 8: while-else — else runs when condition becomes False
n = 3
log = []
while n > 0:
    log.append(n)
    n -= 1
else:
    log.append(0)
assert log == [3, 2, 1, 0], f"while-else = {log!r}"

# Rule 9: nested for loops with break only exits inner
found_pair = None
for outer in range(5):
    for inner in range(5):
        if outer + inner == 6:
            found_pair = (outer, inner)
            break
    if found_pair:
        break
assert found_pair == (2, 4), f"nested break = {found_pair!r}"

# Rule 10: ternary expression (conditional expression)
for v, expected in [(0, "falsy"), (1, "truthy"), ("", "falsy"), ("x", "truthy")]:
    result = "truthy" if v else "falsy"
    assert result == expected, f"ternary({v!r}) = {result!r}"

# Rule 11: pass is a no-op
for _i in range(3):
    pass
assert _i == 2, f"pass loop i = {_i!r}"  # type: ignore[possibly-undefined]

print("behavior OK")
