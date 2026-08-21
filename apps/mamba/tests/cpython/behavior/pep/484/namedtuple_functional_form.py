# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "namedtuple_functional_form"
# subject = "typing.NamedTuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.NamedTuple: the functional NamedTuple('Employee',[('name',str),('age',int)]) syntax builds a tuple subclass with named access and metadata: Employee('Nick',25).name=='Nick', __name__=='Employee', _fields==('name','age'); mixing the list form with keywords raises TypeError"""
from typing import NamedTuple

# Functional NamedTuple via the (name, [(field, type), ...]) syntax.
Employee = NamedTuple("Employee", [("name", str), ("age", int)])
e = Employee("Nick", 25)
assert isinstance(e, tuple)
assert e.name == "Nick"
assert Employee.__name__ == "Employee"
assert Employee._fields == ("name", "age")

# Mixing the list form with keywords is rejected.
try:
    NamedTuple("Bad", [("x", int)], y=str)
    raise AssertionError("expected TypeError")
except TypeError:
    pass

print("namedtuple_functional_form OK")
