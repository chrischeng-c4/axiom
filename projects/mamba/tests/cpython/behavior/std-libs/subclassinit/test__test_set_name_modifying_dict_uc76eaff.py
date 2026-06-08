# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_set_name_modifying_dict_uc76eaff"
# subject = "cpython.test_subclassinit.Test.test_set_name_modifying_dict"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import types
notified = []

class Descriptor:

    def __set_name__(self, owner, name):
        setattr(owner, name + 'x', None)
        notified.append(name)

class A:
    a = Descriptor()
    b = Descriptor()
    c = Descriptor()
    d = Descriptor()
    e = Descriptor()
assert notified == ['a', 'b', 'c', 'd', 'e']

print("Test::test_set_name_modifying_dict: ok")
