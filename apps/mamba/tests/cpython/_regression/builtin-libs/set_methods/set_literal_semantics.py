# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""set_methods: set-literal evaluation order and numeric deduplication."""

# A set literal equals the constructor over the same elements.
assert {1, 2, 3} == set([1, 2, 3])

# Elements of a set literal are evaluated left to right.
events = []
def record(x):
    events.append(x)
    return x
_ = {record(1), record(2), record(3)}
assert events == [1, 2, 3]

# 1, 1.0 and True are all equal and hash alike, so the literal keeps one.
collapsed = {1, 1.0, True}
assert len(collapsed) == 1

# The element kept is the first one inserted: int 1 here.
stored = collapsed.pop()
assert type(stored) is int
assert stored == 1

# Insertion order decides which equal value survives: float wins when first.
float_first = {1.0, 1, True}
kept = float_first.pop()
assert type(kept) is float
assert kept == 1.0

# 0, 0.0 and False likewise collapse to a single element.
zeros = {0, 0.0, False}
assert len(zeros) == 1

# Distinct numeric values are all retained.
assert {1, 2, 3.0, 4} == {1, 2, 3, 4}

print("set_literal_semantics OK")
