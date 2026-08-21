# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_slots_getattr"
# subject = "cpython321.lang_slots_getattr"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_slots_getattr.py"
# status = "filled"
# ///
"""cpython321.lang_slots_getattr: execute CPython 3.12 seed lang_slots_getattr"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for class-level attribute control
# surfaces.
# Surface: __slots__ on a class restricts the legal attribute names
# on instances and raises AttributeError on unlisted attrs; __slots__
# composes through inheritance (subclass adds to allowed names);
# __getattr__ is invoked as a fallback when normal lookup misses;
# hasattr, getattr (with and without default), setattr, delattr
# control instance attributes through the formal API.
_ledger: list[int] = []


# A slotted class only allows attribute names listed in __slots__
class _Point:
    __slots__ = ("x", "y")
    def __init__(self, x, y):
        self.x = x
        self.y = y

p = _Point(1, 2)
# Reading the declared slots works
assert p.x == 1; _ledger.append(1)
assert p.y == 2; _ledger.append(1)
# Writing an unlisted attribute raises AttributeError
raised = False
try:
    p.z = 99
except AttributeError:
    raised = True
assert raised; _ledger.append(1)


# Subclass adds its own __slots__ — both layers' slots remain accessible
class _Vec3(_Point):
    __slots__ = ("z",)
    def __init__(self, x, y, z):
        super().__init__(x, y)
        self.z = z

v = _Vec3(1, 2, 3)
assert v.x == 1; _ledger.append(1)
assert v.y == 2; _ledger.append(1)
assert v.z == 3; _ledger.append(1)
# Writing an attribute not in either __slots__ still raises
raised2 = False
try:
    v.w = 99
except AttributeError:
    raised2 = True
assert raised2; _ledger.append(1)


# __getattr__ runs as the fallback when normal attribute lookup misses
class _Lazy:
    def __init__(self):
        self.real = "real_value"
    def __getattr__(self, name):
        return f"missing:{name}"

ll = _Lazy()
# Normal lookup hits the instance attribute and returns directly
assert ll.real == "real_value"; _ledger.append(1)
# A missing attribute triggers __getattr__
assert ll.something == "missing:something"; _ledger.append(1)
assert ll.another == "missing:another"; _ledger.append(1)


# hasattr / getattr / setattr / delattr operate through the formal API
class _Holder:
    pass

oh = _Holder()
oh.x = 5
# hasattr probes existence
assert hasattr(oh, "x") == True; _ledger.append(1)
assert hasattr(oh, "y") == False; _ledger.append(1)
# getattr without default returns the value
assert getattr(oh, "x") == 5; _ledger.append(1)
# getattr with default returns the default on miss
assert getattr(oh, "y", "default") == "default"; _ledger.append(1)
# setattr writes the attribute
setattr(oh, "z", 99)
assert oh.z == 99; _ledger.append(1)
# delattr removes the attribute
delattr(oh, "x")
assert hasattr(oh, "x") == False; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_slots_getattr {sum(_ledger)} asserts")
