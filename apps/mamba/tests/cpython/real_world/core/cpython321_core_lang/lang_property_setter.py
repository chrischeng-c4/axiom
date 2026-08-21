# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_property_setter"
# subject = "cpython321.lang_property_setter"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_property_setter.py"
# status = "filled"
# ///
"""cpython321.lang_property_setter: execute CPython 3.12 seed lang_property_setter"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the @<name>.setter / @<name>.deleter
# attribute-assignment hooks on a @property.
# Surface: assignment to a property name routes through the .setter
# function (not bypassing it), the setter can raise on invalid input,
# the underlying storage attribute is reflected back through the
# getter, derived properties recompute on each read after the set.
_ledger: list[int] = []

class Temp:
    def __init__(self, c):
        self._c = c
    @property
    def celsius(self):
        return self._c
    @celsius.setter
    def celsius(self, value):
        if value < -273.15:
            raise ValueError("below absolute zero")
        self._c = value
    @property
    def fahrenheit(self):
        return self._c * 9 / 5 + 32

t = Temp(0)
# Initial read flows through the getter
assert t.celsius == 0; _ledger.append(1)
assert t.fahrenheit == 32.0; _ledger.append(1)

# Setting through the property name invokes the .setter
t.celsius = 100
assert t.celsius == 100; _ledger.append(1)
# Derived property recomputes from the updated underlying storage
assert t.fahrenheit == 212.0; _ledger.append(1)

# A setter that raises propagates the exception, leaving the storage
# unchanged
caught = ""
try:
    t.celsius = -300
except ValueError as e:
    caught = str(e)
assert caught == "below absolute zero"; _ledger.append(1)
# Storage is still the last good value (100)
assert t.celsius == 100; _ledger.append(1)

# Mutating directly via the underlying attribute also flows back
# through the getter (the property is a thin wrapper)
t._c = 50
assert t.celsius == 50; _ledger.append(1)
assert t.fahrenheit == 122.0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_property_setter {sum(_ledger)} asserts")
