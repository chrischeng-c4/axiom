# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""dict_methods: binary (non-augmented) `|` on a dict SUBCLASS operand
(CPython 3.12 oracle).

Sibling of dict_subclass_augmented_assign.py (#1026): unlike `|=`, which
mutates the receiver in place and preserves its subclass type, the ordinary
binary `__or__` operator always constructs a brand-new PLAIN dict -- even
when one or both operands are a dict-subclass instance, and regardless of
which side (LHS/RHS) the subclass is on.
"""


class DS(dict):
    pass


# Subclass | plain -> plain dict.
r = DS({"a": 1}) | {"b": 2}
assert type(r) is dict
assert r == {"a": 1, "b": 2}

# Plain | subclass (reverse operand) -> plain dict.
r = {"a": 1} | DS({"b": 2})
assert type(r) is dict
assert r == {"a": 1, "b": 2}

# Subclass | subclass -> plain dict.
r = DS({"a": 1}) | DS({"b": 2})
assert type(r) is dict
assert r == {"a": 1, "b": 2}

print("dict_subclass_binop_drops_subclass OK")
