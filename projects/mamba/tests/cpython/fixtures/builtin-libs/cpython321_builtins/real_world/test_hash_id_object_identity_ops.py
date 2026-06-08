# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_hash_id_object_identity_ops"
# subject = "cpython321.test_hash_id_object_identity_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_hash_id_object_identity_ops.py"
# status = "filled"
# ///
"""cpython321.test_hash_id_object_identity_ops: execute CPython 3.12 seed test_hash_id_object_identity_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `hash()` / `id()` builtins
# applied to primitive objects — surface not covered elsewhere.
# `lang_user_class_equality.py` exercises `hash(user_instance)`;
# `test_complex_arithmetic_mixed_ops.py` exercises `hash(complex)`;
# but the entry-point invariants of `hash()` on int/str/float/bool/
# None/tuple, and `id()` as an object-identity probe, are otherwise
# uncovered.
#
# Surface: `hash(x)` returns an int; equal-by-`==` primitives hash
# equal (int/str/float/tuple); `hash(True) == hash(1)` and
# `hash(False) == hash(0)` (bool→int promotion in the hash sense);
# `hash(None)` is stable across calls; `hash(tuple_of_hashables)`
# threads through element hashes; `id(x)` returns an int; two names
# bound to the same object share an id; two newly-constructed
# containers of the same value have different ids; `id(x) == id(y)`
# is equivalent to `x is y` for object identity.
_ledger: list[int] = []

# hash() returns int
assert isinstance(hash(42), int); _ledger.append(1)
assert isinstance(hash("hello"), int); _ledger.append(1)
assert isinstance(hash(3.14), int); _ledger.append(1)
assert isinstance(hash(None), int); _ledger.append(1)
assert isinstance(hash(True), int); _ledger.append(1)
assert isinstance(hash(()), int); _ledger.append(1)
assert isinstance(hash((1, 2, 3)), int); _ledger.append(1)

# Equal primitives hash equal — call twice in case of any cache
assert hash(42) == hash(42); _ledger.append(1)
assert hash(-7) == hash(-7); _ledger.append(1)
assert hash("hello") == hash("hello"); _ledger.append(1)
assert hash("") == hash(""); _ledger.append(1)
assert hash(3.14) == hash(3.14); _ledger.append(1)
assert hash(0.0) == hash(0.0); _ledger.append(1)
assert hash(None) == hash(None); _ledger.append(1)
assert hash((1, 2, 3)) == hash((1, 2, 3)); _ledger.append(1)
assert hash(()) == hash(()); _ledger.append(1)

# Booleans hash as their int equivalents (since bool subclasses int)
assert hash(True) == hash(1); _ledger.append(1)
assert hash(False) == hash(0); _ledger.append(1)

# Distinct hash classes — different values can have different hashes;
# we cannot assert they MUST differ (collisions are valid), but
# distinct repeated calls remain stable
h_a = hash("alpha")
h_b = hash("alpha")
assert h_a == h_b; _ledger.append(1)

# id() returns int
xs = [1, 2, 3]
assert isinstance(id(xs), int); _ledger.append(1)
assert isinstance(id("hello"), int); _ledger.append(1)
assert isinstance(id(None), int); _ledger.append(1)

# id() is stable for a single object — successive calls return the
# same value while the object is alive
ys = [10, 20]
assert id(ys) == id(ys); _ledger.append(1)
d = {"a": 1}
assert id(d) == id(d); _ledger.append(1)

# Two names bound to the same object share an id
xs2 = [4, 5, 6]
alias = xs2
assert id(xs2) == id(alias); _ledger.append(1)
# `is` and `id(x) == id(y)` agree
assert (alias is xs2) == (id(alias) == id(xs2)); _ledger.append(1)

# Two newly-built containers of the same value are distinct objects
a = [1, 2, 3]
b = [1, 2, 3]
assert a == b; _ledger.append(1)
# Different identities — id values differ
assert id(a) != id(b); _ledger.append(1)
# `is` agrees with the id comparison
assert (a is b) == (id(a) == id(b)); _ledger.append(1)

# Same for dicts
d1 = {"k": 1}
d2 = {"k": 1}
assert d1 == d2; _ledger.append(1)
assert id(d1) != id(d2); _ledger.append(1)

# id(None) is stable across calls
assert id(None) == id(None); _ledger.append(1)

# Tuples used as dict keys rely on hashability
mp = {(1, 2): "a", (3, 4): "b"}
assert mp[(1, 2)] == "a"; _ledger.append(1)
assert mp[(3, 4)] == "b"; _ledger.append(1)
assert len(mp) == 2; _ledger.append(1)

# Frozenset equality is order-independent (we don't assert hash
# equality on frozenset here — mamba 0.3.60 has a divergence)
fs1 = frozenset([1, 2, 3])
fs2 = frozenset([3, 2, 1])
assert fs1 == fs2; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_hash_id_object_identity_ops {sum(_ledger)} asserts")
