# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `repr(frozenset())` (the empty case) was emitting `frozenset({})` — a stray
# pair of curly braces left over from the non-empty `frozenset({1, 2})` form.
# CPython prints just `frozenset()` for the empty frozenset, matching `set()`
# (which already worked correctly). The fix splits the empty branch out in
# all three repr paths: `runtime/string_ops.rs::value_to_string` (used by
# `str(...)` and the in-list/dict repr), and the two `mb_print` /
# `print_repr` arms in `runtime/builtins.rs`.

# Empty frozenset — the headline bug.
print(repr(frozenset()))                   # frozenset()
print(frozenset())                          # frozenset()

# Non-empty — must not regress.
print(repr(frozenset([1])))                 # frozenset({1})

# Inside a container — exercises the value_to_string path.
print([frozenset()])                        # [frozenset()]
print((frozenset(), frozenset([1])))        # (frozenset(), frozenset({1}))

# Empty set still says `set()` (the parallel that frozenset() now matches).
print(repr(set()))                          # set()
