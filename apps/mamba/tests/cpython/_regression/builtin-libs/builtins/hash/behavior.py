"""Behavior contract for builtins.hash.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: hash of equal objects is equal
assert hash(1) == hash(1), "hash(1) not stable"
assert hash("hello") == hash("hello"), "hash('hello') not stable"

# Rule 2: int hash — small ints hash to themselves
assert hash(0) == 0, f"hash(0) = {hash(0)!r}"
assert hash(1) == 1, f"hash(1) = {hash(1)!r}"
assert hash(-1) == -2, f"hash(-1) = {hash(-1)!r}"  # CPython maps -1 → -2
assert hash(-2) == -2, f"hash(-2) = {hash(-2)!r}"  # CPython: -2 stays -2

# Rule 3: bool hash matches int (since bool is subtype of int)
assert hash(True) == hash(1), f"hash(True) = {hash(True)!r}"
assert hash(False) == hash(0), f"hash(False) = {hash(False)!r}"

# Rule 4: float hash — float equal to int has same hash
assert hash(1.0) == hash(1), f"hash(1.0) == hash(1) failed"
assert hash(0.0) == hash(0), f"hash(0.0) == hash(0) failed"

# Rule 5: None has a stable hash
h = hash(None)
assert isinstance(h, int), f"hash(None) not int: {h!r}"
assert hash(None) == h, "hash(None) not stable"

# Rule 6: tuple hash — same tuple same hash
assert hash((1, 2, 3)) == hash((1, 2, 3)), "tuple hash not stable"

# Rule 7: unhashable types raise TypeError
for _obj in ([], {}, set()):
    _raised = False
    try:
        hash(_obj)
    except TypeError:
        _raised = True
    assert _raised, f"hash({_obj!r}) did not raise TypeError"

# Rule 8: custom __hash__
class _Fixed:
    def __hash__(self):
        return 42
assert hash(_Fixed()) == 42, f"custom __hash__ = {hash(_Fixed())!r}"

# Rule 9: custom __hash__ returning non-int raises TypeError
class _BadHash:
    def __hash__(self):
        return 1.5  # type: ignore[return-value]
_raised = False
try:
    hash(_BadHash())
except TypeError:
    _raised = True
assert _raised, "non-int __hash__ did not raise TypeError"

print("behavior OK")
