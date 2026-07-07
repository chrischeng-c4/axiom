# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""list_methods: binary (non-augmented) `+`/`*` on a list SUBCLASS operand
(CPython 3.12 oracle).

Sibling of list_subclass_augmented_assign.py (#1026): unlike `+=`/`*=`, which
mutate the receiver in place and preserve its subclass type, the ordinary
binary `__add__`/`__mul__` operators always construct a brand-new PLAIN list
-- even when one or both operands are a list-subclass instance, and
regardless of which side (LHS/RHS) the subclass is on.
"""


class LS(list):
    pass


# Subclass + plain -> plain list.
r = LS([1]) + [2]
assert type(r) is list
assert r == [1, 2]

# Plain + subclass (reverse operand) -> plain list.
r = [1] + LS([2])
assert type(r) is list
assert r == [1, 2]

# Subclass + subclass -> plain list (neither operand's type survives).
r = LS([1]) + LS([2])
assert type(r) is list
assert r == [1, 2]

# Subclass * int -> plain list.
r = LS([1, 2]) * 2
assert type(r) is list
assert r == [1, 2, 1, 2]

# int * subclass (reverse operand) -> plain list.
r = 2 * LS([1, 2])
assert type(r) is list
assert r == [1, 2, 1, 2]

print("list_subclass_binop_drops_subclass OK")
