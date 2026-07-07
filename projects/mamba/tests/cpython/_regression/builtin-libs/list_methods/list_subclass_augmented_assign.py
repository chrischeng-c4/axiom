# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""list_methods: augmented assignment on a list SUBCLASS (CPython 3.12 oracle).

Regression for #1026: `LS(list) += ...` / `LS(list) *= ...` used to silently
yield None instead of mutating in place. list.__iadd__/__imul__ mutate the
receiver in place, so both the value AND the object identity/subclass type
must be preserved -- unlike a subclass's binary `+`/`*` (see
list_subclass_binop_drops_subclass.py sibling), which always construct a new
plain list.
"""


class LS(list):
    pass


# += extends in place: identity and subclass type both preserved.
l = LS([1])
lid = id(l)
l += [2, 3]
assert type(l) is LS
assert list(l) == [1, 2, 3]
assert id(l) == lid

# *= repeats in place: identity and subclass type both preserved.
l2 = LS([1, 2])
l2id = id(l2)
l2 *= 3
assert type(l2) is LS
assert list(l2) == [1, 2, 1, 2, 1, 2]
assert id(l2) == l2id

# *= by zero/negative clears in place (still the same object).
l3 = LS([1, 2, 3])
l3id = id(l3)
l3 *= 0
assert type(l3) is LS
assert list(l3) == []
assert id(l3) == l3id

# A user-defined __iadd__/__imul__ override still takes priority over the
# builtin in-place mutate.
class LSOverrideAdd(list):
    def __iadd__(self, other):
        self.append("OVERRIDE_ADD")
        return self


lo = LSOverrideAdd([1])
lo += [2]
assert list(lo) == [1, "OVERRIDE_ADD"]


class LSOverrideMul(list):
    def __imul__(self, other):
        self.append("OVERRIDE_MUL")
        return self


lm = LSOverrideMul([1])
lm *= 5
assert list(lm) == [1, "OVERRIDE_MUL"]

# A plain (non-subclass) list on the LHS with a subclass instance as the RHS
# operand still extends correctly (reverse-operand form).
plain = [1]
plain += LS([2, 3])
assert plain == [1, 2, 3]
assert type(plain) is list

print("list_subclass_augmented_assign OK")
