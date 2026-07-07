# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""dict_methods: augmented assignment on a dict SUBCLASS (CPython 3.12 oracle).

Regression for #1026: `DS(dict) |= ...` used to silently yield None instead
of mutating in place. dict.__ior__ mutates the receiver in place (like
list's __iadd__/__imul__), so both the value AND the object identity/subclass
type must be preserved -- unlike a subclass's binary `|` (see
dict_subclass_binop_drops_subclass.py sibling), which always constructs a new
plain dict.
"""


class DS(dict):
    pass


d = DS({"a": 1})
did = id(d)
d |= {"b": 2}
assert type(d) is DS
assert len(d) == 2
assert sorted(d.keys()) == ["a", "b"]
assert d["a"] == 1 and d["b"] == 2
assert id(d) == did

# |= also accepts an iterable of key/value pairs, same as dict.update().
d2 = DS({"a": 1})
d2id = id(d2)
d2 |= [("b", 2), ("c", 3)]
assert type(d2) is DS
assert sorted(d2.keys()) == ["a", "b", "c"]
assert id(d2) == d2id

# A user-defined __ior__ override still takes priority over the builtin
# in-place mutate.
class DSOverride(dict):
    def __ior__(self, other):
        self["marker"] = True
        return self


do = DSOverride({"a": 1})
do |= {"b": 2}
assert do["a"] == 1
assert do["marker"] is True
assert "b" not in do

# A plain (non-subclass) dict on the LHS with a subclass instance as the RHS
# operand still merges correctly (reverse-operand form).
plain = {"a": 1}
plain |= DS({"b": 2})
assert plain == {"a": 1, "b": 2}
assert type(plain) is dict

print("dict_subclass_augmented_assign OK")
