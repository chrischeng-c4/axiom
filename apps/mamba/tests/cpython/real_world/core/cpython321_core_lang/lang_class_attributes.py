# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_class_attributes"
# subject = "cpython321.lang_class_attributes"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_class_attributes.py"
# status = "filled"
# ///
"""cpython321.lang_class_attributes: execute CPython 3.12 seed lang_class_attributes"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for class-attribute vs.
# instance-attribute semantics.
# Surface: a class-level attribute is shared across every instance;
# an instance-level attribute set in __init__ is per-instance;
# methods read self.attr — which finds the instance attr if present,
# else the class attr; assigning to instance.attr creates an
# instance-level shadowing attribute without disturbing the class
# attr seen by other instances; assigning to Class.attr is visible
# from every instance that has not shadowed it.
_ledger: list[int] = []


# Class-level counter, instance-level counter
class _Counter:
    count_class = 0  # class attr — shared across all instances

    def __init__(self):
        self.count_inst = 0  # instance attr — one per instance

    def inc(self):
        self.count_inst += 1
        _Counter.count_class += 1


a = _Counter()
b = _Counter()
a.inc(); a.inc(); b.inc()

# Each instance has its own count_inst
assert a.count_inst == 2; _ledger.append(1)
assert b.count_inst == 1; _ledger.append(1)
# The class-level count was incremented by every inc() call
assert _Counter.count_class == 3; _ledger.append(1)
# Class attr is visible through an instance
assert a.count_class == 3; _ledger.append(1)
assert b.count_class == 3; _ledger.append(1)

# Assigning to Class.attr updates the shared value
_Counter.count_class = 100
assert a.count_class == 100; _ledger.append(1)
assert b.count_class == 100; _ledger.append(1)


# Class attr is the default; assigning to instance.attr shadows
class _Box:
    width = 10  # default width

    def area(self, height):
        return self.width * height


bb = _Box()
# Reads the class default
assert bb.width == 10; _ledger.append(1)
assert bb.area(5) == 50; _ledger.append(1)

# Assigning to bb.width creates an instance attr that shadows the class
bb.width = 20
assert bb.width == 20; _ledger.append(1)
assert bb.area(5) == 100; _ledger.append(1)

# A fresh instance still sees the class default — the shadow is
# per-instance only
cc = _Box()
assert cc.width == 10; _ledger.append(1)
assert cc.area(5) == 50; _ledger.append(1)

# bb's instance attr is unaffected by reads on cc
assert bb.width == 20; _ledger.append(1)


# Class attr lookup falls through to the class when no instance attr
class _Defaults:
    name = "default"


d1 = _Defaults()
d2 = _Defaults()
# Both see the class-level default
assert d1.name == "default"; _ledger.append(1)
assert d2.name == "default"; _ledger.append(1)
# Setting on one instance doesn't affect the other
d1.name = "override"
assert d1.name == "override"; _ledger.append(1)
assert d2.name == "default"; _ledger.append(1)
# Changing the class default is visible to instances that haven't shadowed
_Defaults.name = "new-default"
assert d1.name == "override"; _ledger.append(1)
assert d2.name == "new-default"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_class_attributes {sum(_ledger)} asserts")
