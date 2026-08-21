"""Surface contract for builtins.hash.

# type-regime: monomorphic

Probes: name presence, callable, returns int for hashable types,
raises TypeError for unhashable types.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "hash"), "builtins.hash missing"
assert builtins.hash is hash, "builtins.hash is hash divergence"
assert callable(builtins.hash), "builtins.hash not callable"

# hash returns int for hashable types
assert isinstance(hash(42), int), "hash(42) not int"
assert isinstance(hash(3.14), int), "hash(3.14) not int"
assert isinstance(hash("hello"), int), "hash('hello') not int"
assert isinstance(hash(b"data"), int), "hash(b'data') not int"
assert isinstance(hash((1, 2)), int), "hash((1,2)) not int"
assert isinstance(hash(None), int), "hash(None) not int"
assert isinstance(hash(True), int), "hash(True) not int"
assert isinstance(hash(frozenset()), int), "hash(frozenset()) not int"

# hash raises TypeError for unhashable types
for _obj in ([], {}, set()):
    _raised = False
    try:
        hash(_obj)
    except TypeError:
        _raised = True
    assert _raised, f"hash({_obj!r}) did not raise TypeError"

print("surface OK")
