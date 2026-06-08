"""Behavior contract for builtins.id.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: id returns non-negative int
assert id(None) >= 0, f"id(None) = {id(None)!r}"
assert id(0) >= 0, f"id(0) = {id(0)!r}"
assert id("") >= 0, f"id('') = {id('')!r}"
assert id([]) >= 0, f"id([]) = {id([])!r}"

# Rule 2: id of the same object is stable
x = [1, 2, 3]
id_x = id(x)
assert id(x) == id_x, "id(x) changed on second call"
x.append(4)  # mutation doesn't change id
assert id(x) == id_x, "id(x) changed after mutation"

# Rule 3: distinct objects have distinct ids
a = object()
b = object()
assert id(a) != id(b), "distinct objects should have different ids"

# Rule 4: None is a singleton — id stable
none_id = id(None)
assert id(None) == none_id, "id(None) not stable"

# Rule 5: is operator and id are equivalent
a = object()
b = a
assert (a is b) == (id(a) == id(b)), "is <=> id equality"
c = object()
assert (a is c) == (id(a) == id(c)), "is <=> id inequality"

# Rule 6: id is an int (not subclass)
assert type(id(None)) is int, f"type(id(None)) = {type(id(None)).__name__!r}"

# Rule 7: id works on all builtin types
for obj in [42, 3.14, "hi", b"data", [], (), {}, set(), frozenset(), True, None]:
    assert isinstance(id(obj), int), f"id({obj!r}) not int"

print("behavior OK")
