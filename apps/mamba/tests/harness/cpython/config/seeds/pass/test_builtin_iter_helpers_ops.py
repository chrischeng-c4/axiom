# Operational AssertionPass seed for iterator-helper built-ins.
# Surface:
#   • enumerate(iter) — default start=0 produces (index, item) pairs;
#   • enumerate(iter, start=N) — explicit start offset;
#   • enumerate over an empty iterable — yields nothing;
#   • zip(iter, iter, strict=True) — succeeds when lengths match;
#   • zip(... strict=True) — raises ValueError when lengths differ;
#   • zip() / zip([], ...) — empty / short-circuit empty;
#   • map(callable, []) — empty iterable;
#   • filter(bool, iter) — keep truthy items only;
#   • filter(None, []) and filter(predicate, []) — empty iterables;
#   • reversed on list / str / empty — yields items right-to-left;
#   • sum(iter, start) — explicit start offset for accumulation;
#   • min(iter, default=) / max(iter, default=) — default for empty;
#   • min(iter, key=) / max(iter, key=) — comparison key;
#   • sorted(iter, key=, reverse=) — combined key + reverse.
#
# Surfaces deliberately NOT exercised here — mamba 0.3.60 disagrees:
#   • zip([x]) single-iterable form (mamba yields wrong shape);
#   • map(fn, iter1, iter2) multi-iterable (mamba returns wrong);
#   • any/all over generator expressions (mamba mis-evaluates empty).
# Those gaps move to focused spec/ or fail/ seeds.
_ledger: list[int] = []

# enumerate — default start=0
assert list(enumerate(["a", "b", "c"])) == [(0, "a"), (1, "b"), (2, "c")]; _ledger.append(1)
assert list(enumerate("xy")) == [(0, "x"), (1, "y")]; _ledger.append(1)

# enumerate — explicit start
assert list(enumerate(["a", "b"], start=10)) == [(10, "a"), (11, "b")]; _ledger.append(1)
assert list(enumerate("xyz", start=-1)) == [(-1, "x"), (0, "y"), (1, "z")]; _ledger.append(1)
assert list(enumerate(["x"], start=100)) == [(100, "x")]; _ledger.append(1)

# enumerate — empty
assert list(enumerate([])) == []; _ledger.append(1)
assert list(enumerate([], 5)) == []; _ledger.append(1)

# zip(strict=True) — equal length succeeds
assert list(zip([1, 2, 3], ["a", "b", "c"], strict=True)) == [(1, "a"), (2, "b"), (3, "c")]; _ledger.append(1)
assert list(zip([], [], strict=True)) == []; _ledger.append(1)

# zip(strict=True) — length mismatch raises ValueError
try:
    _ = list(zip([1, 2], ["a", "b", "c"], strict=True))
    raise AssertionError("zip strict must raise on mismatch")
except ValueError:
    _ledger.append(1)

try:
    _ = list(zip([1, 2, 3], ["a"], strict=True))
    raise AssertionError("zip strict must raise on mismatch (reverse)")
except ValueError:
    _ledger.append(1)

# zip() — no args yields empty
assert list(zip()) == []; _ledger.append(1)

# zip([], non-empty) — short-circuit empty
assert list(zip([], [1, 2])) == []; _ledger.append(1)
assert list(zip([1, 2, 3], [])) == []; _ledger.append(1)

# map — empty iterable
assert list(map(str, [])) == []; _ledger.append(1)
assert list(map(abs, [])) == []; _ledger.append(1)

# filter — truthy retain
assert list(filter(bool, [0, 1, 0, 2, 0, 3])) == [1, 2, 3]; _ledger.append(1)
assert list(filter(None, [0, 1, "", "a", None, [], [1]])) == [1, "a", [1]]; _ledger.append(1)

# filter — empty
assert list(filter(None, [])) == []; _ledger.append(1)
assert list(filter(bool, [])) == []; _ledger.append(1)

# reversed — list / str / empty
assert list(reversed([1, 2, 3])) == [3, 2, 1]; _ledger.append(1)
assert list(reversed([])) == []; _ledger.append(1)
assert list(reversed("abc")) == ["c", "b", "a"]; _ledger.append(1)
assert list(reversed((10, 20, 30))) == [30, 20, 10]; _ledger.append(1)

# sum with explicit start
assert sum([1, 2, 3], 100) == 106; _ledger.append(1)
assert sum([1.5, 2.5], 0.0) == 4.0; _ledger.append(1)
assert sum([], 42) == 42; _ledger.append(1)
assert sum([1, 2, 3]) == 6; _ledger.append(1)

# min/max with default for empty
assert min([], default=99) == 99; _ledger.append(1)
assert max([], default=-1) == -1; _ledger.append(1)
assert min([1, 2, 3], default=99) == 1; _ledger.append(1)
assert max([1, 2, 3], default=99) == 3; _ledger.append(1)

# min/max with key — distinct key values to dodge mamba's tie-break
# direction (CPython picks first on ties, mamba 0.3.60 picks last).
assert min(["aa", "b", "ccc"], key=len) == "b"; _ledger.append(1)
assert max(["aa", "b", "ccc"], key=len) == "ccc"; _ledger.append(1)
assert max([1, -3, 2, -5], key=abs) == -5; _ledger.append(1)
assert min([-1, 2, -3, 4], key=abs) == -1; _ledger.append(1)

# sorted with key + reverse combined
assert sorted([3, 1, 2]) == [1, 2, 3]; _ledger.append(1)
assert sorted([3, 1, 2], reverse=True) == [3, 2, 1]; _ledger.append(1)
assert sorted(["bbb", "a", "cc"], key=len) == ["a", "cc", "bbb"]; _ledger.append(1)
assert sorted(["bbb", "a", "cc"], key=len, reverse=True) == ["bbb", "cc", "a"]; _ledger.append(1)
assert sorted([1, -2, 3, -4], key=abs) == [1, -2, 3, -4]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_builtin_iter_helpers_ops {sum(_ledger)} asserts")
