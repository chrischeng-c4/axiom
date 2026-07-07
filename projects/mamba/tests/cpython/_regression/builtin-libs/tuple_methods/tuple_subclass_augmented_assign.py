# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""tuple_methods: augmented assignment on a tuple SUBCLASS (CPython 3.12 oracle).

Regression for #1026: `TS(tuple) += ...` used to silently yield None. tuple
has no in-place mutating dunder at all (it's immutable), so CPython's `+=`
falls through to the ordinary `__add__` binary operator, which always
constructs a brand-new PLAIN tuple -- the subclass type is dropped and object
identity is never preserved, unlike list/dict's in-place augmented ops (see
the list_methods/dict_methods siblings).
"""


class TS(tuple):
    pass


t = TS((1,))
tid = id(t)
t += (2, 3)
assert type(t) is tuple, f"expected plain tuple, got {type(t)}"
assert t == (1, 2, 3)
assert id(t) != tid, "tuple += must allocate a new object (immutable)"

# Reverse-operand form: plain tuple LHS, subclass instance RHS.
plain = (1,)
plain += TS((2, 3))
assert type(plain) is tuple
assert plain == (1, 2, 3)

# *= likewise has no in-place tuple dunder and drops the subclass too.
t2 = TS((1, 2))
t2 *= 2
assert type(t2) is tuple
assert t2 == (1, 2, 1, 2)

print("tuple_subclass_augmented_assign OK")
