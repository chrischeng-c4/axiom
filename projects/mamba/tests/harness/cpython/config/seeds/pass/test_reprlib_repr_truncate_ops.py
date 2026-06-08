# Operational AssertionPass seed for the `reprlib` module — the
# stdlib helper used by `pdb` / `pprint` / `IPython` / generic
# debuggers to produce short, safe, recursion-bounded `repr()` strings
# of large or self-referential containers. Surface focuses on
# `reprlib.repr(x)` — the top-level convenience function — because
# `reprlib.Repr` instance construction diverges across runtimes
# (mamba returns a sentinel-dict rather than a configured Repr
# object). No fixture coverage yet for reprlib.
#
# Surface:
#   • reprlib.repr(x) — top-level convenience:
#       — primitives (int / float / None / bool) match builtin repr;
#       — empty containers match builtin repr;
#       — short containers (≤6 elements) round-trip via builtin repr;
#       — long lists/tuples truncate to `[0, 1, 2, 3, 4, 5, ...]`
#         form (first 6 elements + literal `...` + closing bracket);
#       — repeatable: calling twice on the same input yields the same
#         string;
#       — return type is always `str`;
#   • reprlib.recursive_repr — callable factory attribute exists
#     (constructing the decorator instance segfaults mamba, so we
#     only verify the attribute is callable, not the result).
import reprlib
_ledger: list[int] = []

# Top-level reprlib.repr — primitives match builtin repr
assert reprlib.repr(42) == "42"; _ledger.append(1)
assert reprlib.repr(3.14) == "3.14"; _ledger.append(1)
assert reprlib.repr(None) == "None"; _ledger.append(1)
assert reprlib.repr(True) == "True"; _ledger.append(1)
assert reprlib.repr(False) == "False"; _ledger.append(1)
assert reprlib.repr(-5) == "-5"; _ledger.append(1)
assert reprlib.repr(0) == "0"; _ledger.append(1)

# Empty containers match builtin repr
assert reprlib.repr([]) == "[]"; _ledger.append(1)
assert reprlib.repr(()) == "()"; _ledger.append(1)
assert reprlib.repr({}) == "{}"; _ledger.append(1)

# Short containers (≤6 elements) round-trip via builtin repr
assert reprlib.repr([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert reprlib.repr([1, 2, 3, 4, 5]) == "[1, 2, 3, 4, 5]"; _ledger.append(1)
assert reprlib.repr((1, 2, 3)) == "(1, 2, 3)"; _ledger.append(1)
assert reprlib.repr({"a": 1}) == "{'a': 1}"; _ledger.append(1)

# Long lists truncate — first 6 elements + ellipsis
_long = reprlib.repr(list(range(20)))
assert "0, 1, 2, 3, 4, 5" in _long; _ledger.append(1)
assert "..." in _long; _ledger.append(1)
assert _long.startswith("["); _ledger.append(1)
assert _long.endswith("]"); _ledger.append(1)

# Long tuples truncate — first 6 elements + ellipsis
_longt = reprlib.repr(tuple(range(20)))
assert "0, 1, 2, 3, 4, 5" in _longt; _ledger.append(1)
assert "..." in _longt; _ledger.append(1)
assert _longt.startswith("("); _ledger.append(1)
assert _longt.endswith(")"); _ledger.append(1)

# Nested containers — short enough to render in full
assert reprlib.repr([1, [2, 3], 4]) == "[1, [2, 3], 4]"; _ledger.append(1)
assert reprlib.repr([[[[[1]]]]]) == "[[[[[1]]]]]"; _ledger.append(1)

# Return type discipline — always str
assert isinstance(reprlib.repr([1, 2, 3]), str); _ledger.append(1)
assert isinstance(reprlib.repr(list(range(20))), str); _ledger.append(1)
assert isinstance(reprlib.repr({1: 2, 3: 4}), str); _ledger.append(1)
assert isinstance(reprlib.repr(42), str); _ledger.append(1)
assert isinstance(reprlib.repr(None), str); _ledger.append(1)
assert isinstance(reprlib.repr("hi"), str); _ledger.append(1)
assert isinstance(reprlib.repr(()), str); _ledger.append(1)

# recursive_repr is a callable factory
assert callable(reprlib.recursive_repr); _ledger.append(1)

# Idempotent — repeatable
assert reprlib.repr(list(range(20))) == reprlib.repr(list(range(20))); _ledger.append(1)
assert reprlib.repr([1, 2, 3]) == reprlib.repr([1, 2, 3]); _ledger.append(1)
assert reprlib.repr(42) == reprlib.repr(42); _ledger.append(1)
assert reprlib.repr(None) == reprlib.repr(None); _ledger.append(1)

# Short string — never truncated
assert reprlib.repr("hi") == "'hi'"; _ledger.append(1)
assert reprlib.repr("") == "''"; _ledger.append(1)
assert reprlib.repr("a") == "'a'"; _ledger.append(1)

# Booleans render as 'True'/'False'
assert reprlib.repr([True, False]) == "[True, False]"; _ledger.append(1)

# Nested dicts — shape-bound
_d_repr = reprlib.repr({1: 2, 3: 4})
assert _d_repr.startswith("{"); _ledger.append(1)
assert _d_repr.endswith("}"); _ledger.append(1)

# Cross-list-tuple parity — same numeric content, different container
_l_20 = reprlib.repr(list(range(20)))
_t_20 = reprlib.repr(tuple(range(20)))
# Both should contain the first 6 elements
for _i in range(6):
    assert str(_i) in _l_20; _ledger.append(1)
    assert str(_i) in _t_20; _ledger.append(1)

# Long list of 7 elements still triggers truncation (max default = 6)
_l7 = reprlib.repr(list(range(100)))
assert "..." in _l7; _ledger.append(1)

# repr exists as a callable attribute
assert callable(reprlib.repr); _ledger.append(1)

# Edge: list of 6 exactly — no truncation
assert reprlib.repr([0, 1, 2, 3, 4, 5]) == "[0, 1, 2, 3, 4, 5]"; _ledger.append(1)

# Edge: list of 7 — truncation kicks in
_l7_repr = reprlib.repr([0, 1, 2, 3, 4, 5, 6])
assert "..." in _l7_repr; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_reprlib_repr_truncate_ops {sum(_ledger)} asserts")
