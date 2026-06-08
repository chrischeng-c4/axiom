# Operational AssertionPass seed for user-defined `__str__` /
# `__repr__` dispatch through the four entry-points: `str(obj)`,
# `repr(obj)`, the f-string default conversion (`f"{obj}"`), and the
# f-string `!r` conversion (`f"{obj!r}"`); plus the `%s` / `%r`
# C-style formatting operators on a user class.
# Surface not covered by `lang_dunders` / `lang_dunder_protocols`
# (which exercise __call__, __iter__, __bool__, __eq__/lt/gt/le/ge,
# __add__/sub/mul/radd, __getitem__/__setitem__/__len__/__contains__).
# This seed asserts: when a class defines BOTH __str__ and __repr__,
# str()/f-string-default invoke __str__, while repr()/f-string-!r/
# %r invoke __repr__. When a class defines only __repr__, str()
# falls back to __repr__. The f-string default conversion dispatches
# through __str__ same as str(), and the !r conversion through
# __repr__ same as repr().
_ledger: list[int] = []


class _Pt:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def __str__(self):
        return f"Pt({self.x}, {self.y})"
    def __repr__(self):
        return f"<Pt:{self.x},{self.y}>"


p = _Pt(3, 4)

# str(obj) dispatches through __str__
assert str(p) == "Pt(3, 4)"; _ledger.append(1)
# repr(obj) dispatches through __repr__
assert repr(p) == "<Pt:3,4>"; _ledger.append(1)
# f-string default conversion uses __str__
assert f"{p}" == "Pt(3, 4)"; _ledger.append(1)
# f-string !r conversion uses __repr__
assert f"{p!r}" == "<Pt:3,4>"; _ledger.append(1)
# %s formatting uses __str__
assert ("%s" % p) == "Pt(3, 4)"; _ledger.append(1)
# %r formatting uses __repr__
assert ("%r" % p) == "<Pt:3,4>"; _ledger.append(1)

# Return-type invariants — always str
assert isinstance(str(p), str); _ledger.append(1)
assert isinstance(repr(p), str); _ledger.append(1)
assert isinstance(f"{p}", str); _ledger.append(1)
assert isinstance(f"{p!r}", str); _ledger.append(1)

# Class with only __repr__ — str() falls back to __repr__
class _R:
    def __repr__(self):
        return "<R>"

r = _R()
assert repr(r) == "<R>"; _ledger.append(1)
# str() falls back to __repr__ when __str__ isn't defined
assert str(r) == "<R>"; _ledger.append(1)
# f-string default also falls back
assert f"{r}" == "<R>"; _ledger.append(1)
# f-string !r still works
assert f"{r!r}" == "<R>"; _ledger.append(1)

# Class with only __str__ — str()/f-string use __str__
class _S:
    def __str__(self):
        return "S-str"

s = _S()
assert str(s) == "S-str"; _ledger.append(1)
# f-string default uses __str__
assert f"{s}" == "S-str"; _ledger.append(1)
# %s uses __str__
assert ("%s" % s) == "S-str"; _ledger.append(1)

# Concatenation via str() — f-string with multiple values
p2 = _Pt(10, 20)
assert f"{p} and {p2}" == "Pt(3, 4) and Pt(10, 20)"; _ledger.append(1)
assert f"{p!r} vs {p2!r}" == "<Pt:3,4> vs <Pt:10,20>"; _ledger.append(1)

# A list-of-instances default-repr uses __repr__ of each element
# (this is CPython's container behaviour — they call repr inside)
items = [p, p2]
rendered = repr(items)
# Each element rendered via repr (we just assert both repr strings
# appear in the list rendering)
assert "<Pt:3,4>" in rendered; _ledger.append(1)
assert "<Pt:10,20>" in rendered; _ledger.append(1)

# Different instances render their own state
p3 = _Pt(-1, -2)
assert str(p3) == "Pt(-1, -2)"; _ledger.append(1)
assert repr(p3) == "<Pt:-1,-2>"; _ledger.append(1)
# Verify __str__ and __repr__ are independent — same instance, two strings
assert str(p3) != repr(p3); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_user_str_repr_dispatch_ops {sum(_ledger)} asserts")
