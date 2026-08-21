# Operational AssertionPass seed for the matching `itertools` surface
# — finite-iterable combinators (no `count`/`cycle`/`from_iterable`,
# no `initial=`/`repeat=` kwargs which silently diverge on mamba) plus
# the combinatoric helpers (permutations, combinations, product,
# groupby, tee, pairwise).
#
# `itertools` is the canonical iterator library: lazy generators that
# transform one or more iterables. The matching subset between mamba
# and CPython is the FINITE-input combinator family. The infinite-
# generator family (count, cycle, repeat-without-count) and the
# keyword-argument forms (accumulate(initial=), product(repeat=)) are
# split off into the divergence-spec fixture.
#
# Surface in this fixture:
#   • repeat(value, n) — fixed-count repeater;
#   • chain(iter, iter, ...) — concat of fixed iterables;
#   • accumulate(iter)        — running totals (no initial=);
#   • accumulate(iter, fn)    — running totals with a binary fn;
#   • takewhile / dropwhile / filterfalse — predicate-driven cuts;
#   • compress — selector-driven mask;
#   • islice — stop / start,stop / start,stop,step;
#   • starmap — apply a (a, b) -> c fn to argument tuples;
#   • zip_longest — uneven-length zip with default None or fillvalue;
#   • product(iter, iter)   — Cartesian product of 2 iterables;
#   • permutations / combinations / combinations_with_replacement —
#     order- and replacement-aware enumerations;
#   • groupby (consecutive-key) — list-of-(key, list);
#   • tee — n-way iterator split;
#   • pairwise — sliding (n, n+1) pairs.
#
# Behavioral edges that DIVERGE on mamba (count/cycle infinite gens,
# chain.from_iterable empty, accumulate(initial=) broken, product
# (repeat=) broken) are covered in
# `lang_itertools_infinite_kwarg_silent.py`.
import itertools

_ledger: list[int] = []

# 1) repeat(value, count) — fixed-length repeater
assert list(itertools.repeat("x", 3)) == ["x", "x", "x"]; _ledger.append(1)
assert list(itertools.repeat(5, 4)) == [5, 5, 5, 5]; _ledger.append(1)
assert list(itertools.repeat("a", 0)) == []; _ledger.append(1)
assert list(itertools.repeat(None, 2)) == [None, None]; _ledger.append(1)

# 2) chain — concat of fixed iterables
assert list(itertools.chain([1, 2], [3, 4], [5])) == [1, 2, 3, 4, 5]; _ledger.append(1)
assert list(itertools.chain([], [1])) == [1]; _ledger.append(1)
assert list(itertools.chain([1], [])) == [1]; _ledger.append(1)
assert list(itertools.chain([], [])) == []; _ledger.append(1)
assert list(itertools.chain("ab", "cd")) == ["a", "b", "c", "d"]; _ledger.append(1)

# 3) accumulate (no initial= kwarg)
assert list(itertools.accumulate([1, 2, 3, 4])) == [1, 3, 6, 10]; _ledger.append(1)
assert list(itertools.accumulate([5])) == [5]; _ledger.append(1)
assert list(itertools.accumulate([1, 2, 3], lambda a, b: a * b)) == [1, 2, 6]; _ledger.append(1)
assert list(itertools.accumulate([1, 2, 3], lambda a, b: a - b)) == [1, -1, -4]; _ledger.append(1)
# Use a lambda for the running-max (passing `max` directly as a
# binary fn is in the divergence-spec fixture)
assert list(itertools.accumulate([1, 5, 3, 7, 2], lambda a, b: a if a > b else b)) == [1, 5, 5, 7, 7]; _ledger.append(1)

# 4) takewhile / dropwhile / filterfalse
assert list(itertools.takewhile(lambda x: x < 3, [1, 2, 3, 4, 1])) == [1, 2]; _ledger.append(1)
assert list(itertools.takewhile(lambda x: x > 0, [1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
assert list(itertools.takewhile(lambda x: x > 0, [-1, 1, 2])) == []; _ledger.append(1)

assert list(itertools.dropwhile(lambda x: x < 3, [1, 2, 3, 4, 1])) == [3, 4, 1]; _ledger.append(1)
assert list(itertools.dropwhile(lambda x: x < 100, [1, 2, 3])) == []; _ledger.append(1)
assert list(itertools.dropwhile(lambda x: x < 0, [1, 2, 3])) == [1, 2, 3]; _ledger.append(1)

assert list(itertools.filterfalse(lambda x: x < 3, [1, 2, 3, 4, 5])) == [3, 4, 5]; _ledger.append(1)
assert list(itertools.filterfalse(lambda x: x % 2, [1, 2, 3, 4, 5])) == [2, 4]; _ledger.append(1)
assert list(itertools.filterfalse(lambda x: True, [1, 2, 3])) == []; _ledger.append(1)

# 5) compress — element-wise mask
assert list(itertools.compress("ABCDEF", [1, 0, 1, 0, 1, 1])) == ["A", "C", "E", "F"]; _ledger.append(1)
assert list(itertools.compress("ABC", [0, 0, 0])) == []; _ledger.append(1)
assert list(itertools.compress("ABC", [1, 1, 1])) == ["A", "B", "C"]; _ledger.append(1)

# 6) islice — stop / start,stop / start,stop,step
assert list(itertools.islice("ABCDEFG", 3)) == ["A", "B", "C"]; _ledger.append(1)
assert list(itertools.islice("ABCDEFG", 2, 5)) == ["C", "D", "E"]; _ledger.append(1)
assert list(itertools.islice("ABCDEFG", 0, 7, 2)) == ["A", "C", "E", "G"]; _ledger.append(1)
assert list(itertools.islice("ABCDEFG", 1, 7, 2)) == ["B", "D", "F"]; _ledger.append(1)
assert list(itertools.islice([1, 2, 3, 4, 5], 0)) == []; _ledger.append(1)

# 7) starmap — apply a fn-of-args to each argument tuple
assert list(itertools.starmap(lambda a, b: a + b, [(1, 2), (3, 4), (5, 6)])) == [3, 7, 11]; _ledger.append(1)
assert list(itertools.starmap(lambda a, b: a * b, [(2, 3), (4, 5)])) == [6, 20]; _ledger.append(1)
assert list(itertools.starmap(lambda a, b: a - b, [(10, 3)])) == [7]; _ledger.append(1)

# 8) zip_longest — uneven-length zip
assert list(itertools.zip_longest([1, 2, 3], ["a", "b"])) == [(1, "a"), (2, "b"), (3, None)]; _ledger.append(1)
assert list(itertools.zip_longest([1, 2, 3], ["a", "b"], fillvalue="X")) == [(1, "a"), (2, "b"), (3, "X")]; _ledger.append(1)
assert list(itertools.zip_longest([1, 2], [3, 4])) == [(1, 3), (2, 4)]; _ledger.append(1)
assert list(itertools.zip_longest([], [1, 2], fillvalue=0)) == [(0, 1), (0, 2)]; _ledger.append(1)

# 9) product — Cartesian product (2-iterable form, no repeat= kwarg)
assert list(itertools.product([1, 2], "AB")) == [(1, "A"), (1, "B"), (2, "A"), (2, "B")]; _ledger.append(1)
# product across 3+ iterables silently truncates on mamba —
# stays in the divergence-spec fixture
assert list(itertools.product([1, 2], [3])) == [(1, 3), (2, 3)]; _ledger.append(1)
assert list(itertools.product([], [1, 2])) == []; _ledger.append(1)
assert list(itertools.product([1, 2, 3], "XY")) == [(1, "X"), (1, "Y"), (2, "X"), (2, "Y"), (3, "X"), (3, "Y")]; _ledger.append(1)

# 10) permutations / combinations / combinations_with_replacement
assert list(itertools.permutations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 1), (2, 3), (3, 1), (3, 2)]; _ledger.append(1)
assert list(itertools.permutations([1, 2, 3])) == [(1, 2, 3), (1, 3, 2), (2, 1, 3), (2, 3, 1), (3, 1, 2), (3, 2, 1)]; _ledger.append(1)
assert list(itertools.permutations([1, 2])) == [(1, 2), (2, 1)]; _ledger.append(1)
assert list(itertools.permutations([])) == [()]; _ledger.append(1)

assert list(itertools.combinations([1, 2, 3, 4], 2)) == [(1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)]; _ledger.append(1)
assert list(itertools.combinations([1, 2, 3], 3)) == [(1, 2, 3)]; _ledger.append(1)
assert list(itertools.combinations([1, 2], 1)) == [(1,), (2,)]; _ledger.append(1)

assert list(itertools.combinations_with_replacement("AB", 2)) == [("A", "A"), ("A", "B"), ("B", "B")]; _ledger.append(1)
assert list(itertools.combinations_with_replacement([1, 2], 3)) == [(1, 1, 1), (1, 1, 2), (1, 2, 2), (2, 2, 2)]; _ledger.append(1)

# 11) groupby — consecutive-key grouping
_grouped = [(k, list(g)) for k, g in itertools.groupby("AAAABBBCCD")]
assert _grouped == [("A", ["A", "A", "A", "A"]), ("B", ["B", "B", "B"]), ("C", ["C", "C"]), ("D", ["D"])]; _ledger.append(1)

_grouped2 = [(k, list(g)) for k, g in itertools.groupby([1, 1, 2, 2, 3])]
assert _grouped2 == [(1, [1, 1]), (2, [2, 2]), (3, [3])]; _ledger.append(1)

# Non-consecutive key — produces fragments
_grouped3 = [(k, list(g)) for k, g in itertools.groupby("ABA")]
assert _grouped3 == [("A", ["A"]), ("B", ["B"]), ("A", ["A"])]; _ledger.append(1)

# 12) tee — n-way iterator split
_t1, _t2 = itertools.tee([1, 2, 3], 2)
assert list(_t1) == [1, 2, 3]; _ledger.append(1)
assert list(_t2) == [1, 2, 3]; _ledger.append(1)

_tees = itertools.tee("ABC", 3)
assert [list(t) for t in _tees] == [["A", "B", "C"], ["A", "B", "C"], ["A", "B", "C"]]; _ledger.append(1)

# 13) pairwise — sliding (n, n+1) pairs
assert list(itertools.pairwise([1, 2, 3, 4])) == [(1, 2), (2, 3), (3, 4)]; _ledger.append(1)
assert list(itertools.pairwise("ABCD")) == [("A", "B"), ("B", "C"), ("C", "D")]; _ledger.append(1)
assert list(itertools.pairwise([1])) == []; _ledger.append(1)
assert list(itertools.pairwise([])) == []; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_itertools_finite_combinatorics_ops {sum(_ledger)} asserts")
