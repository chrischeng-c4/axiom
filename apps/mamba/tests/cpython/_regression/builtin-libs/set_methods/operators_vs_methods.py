# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""set_methods: operators screen out general iterables; methods accept them.

`s & t` requires both sides to be sets, but `s.intersection(t)` accepts any
iterable. Same split for | / union, - / difference, ^ / symmetric_difference.
This holds for both set and frozenset.
"""

word = "simsalabim"
other = "madagascar"

for thetype in (set, frozenset):
    s = thetype(word)

    # Method form accepts a bare string (an iterable of chars)...
    inter = s.intersection(other)
    union = s.union(other)
    diff = s.difference(other)
    symm = s.symmetric_difference(other)

    # ...and matches the operator form when the RHS is wrapped in a set.
    assert s & set(other) == inter
    assert s & frozenset(other) == inter
    assert s | set(other) == union
    assert s - set(other) == diff
    assert s ^ set(other) == symm

    # The operator form refuses the bare string.
    for op_name, fn in [
        ("&", lambda x: s & x),
        ("|", lambda x: s | x),
        ("-", lambda x: s - x),
        ("^", lambda x: s ^ x),
    ]:
        raised = False
        try:
            fn(other)
        except TypeError:
            raised = True
        assert raised, op_name + " accepted a general iterable"

# The named methods take several positional iterables at once.
assert {1, 2, 3}.union([3, 4], (5,), {6}) == {1, 2, 3, 4, 5, 6}
assert {1, 2, 3, 4}.intersection([2, 3, 4], (3, 4, 5)) == {3, 4}
assert {1, 2, 3, 4}.difference([2], (3,)) == {1, 4}

print("operators_vs_methods OK")
