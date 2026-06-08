# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_property_descriptor"
# subject = "cpython321.lang_property_descriptor"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_property_descriptor.py"
# status = "filled"
# ///
"""cpython321.lang_property_descriptor: execute CPython 3.12 seed lang_property_descriptor"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the @property descriptor.
# Surface: a method decorated with @property is accessed as a bare
# attribute (no parentheses); the property body executes on each
# read; derived properties recompute from underlying state.
class Temperature:
    def __init__(self, celsius):
        self._celsius = celsius

    @property
    def celsius(self):
        return self._celsius

    @property
    def fahrenheit(self):
        return self._celsius * 9 / 5 + 32

    @property
    def kelvin(self):
        return self._celsius + 273.15

_ledger: list[int] = []
t = Temperature(100)
# @property is accessed without parentheses
assert t.celsius == 100; _ledger.append(1)
assert t.fahrenheit == 212.0; _ledger.append(1)
assert t.kelvin == 373.15; _ledger.append(1)
# Another instance recomputes per its own state
t2 = Temperature(0)
assert t2.celsius == 0; _ledger.append(1)
assert t2.fahrenheit == 32.0; _ledger.append(1)
assert t2.kelvin == 273.15; _ledger.append(1)
# Mutate underlying state — derived properties reflect the change
t._celsius = 25
assert t.celsius == 25; _ledger.append(1)
assert t.fahrenheit == 77.0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_property_descriptor {sum(_ledger)} asserts")
