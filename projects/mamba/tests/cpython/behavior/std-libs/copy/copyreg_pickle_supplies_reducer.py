# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "copyreg_pickle_supplies_reducer"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: copyreg.pickle registers a reducer for a class that defines none, so copy succeeds after registration where it failed before"""
import copy
import copyreg


class Registered:
    def __new__(cls, foo):
        obj = object.__new__(cls)
        obj.foo = foo
        return obj


reg = Registered(42)

# Before registration the class has no reducer copy can use.
_failed = False
try:
    copy.copy(reg)
except TypeError:
    _failed = True
assert _failed, "copy should fail before copyreg registration"

# copyreg.pickle supplies the missing reducer; now copy succeeds.
copyreg.pickle(Registered, lambda obj: (Registered, (obj.foo,)), Registered)
reg_c = copy.copy(reg)
assert type(reg_c) is Registered and reg_c.foo == 42 and reg_c is not reg, "copyreg copy"

print("copyreg_pickle_supplies_reducer OK")
