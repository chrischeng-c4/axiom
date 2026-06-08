# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "pickle_getnewargs_reconstructs"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: __getnewargs__ and __getnewargs_ex__ feed the reconstructed instance's __new__ so an int subclass with extra state copies correctly"""
import copy


# __getnewargs__ feeds the reconstructed instance's __new__ (positional).
class NewArgs(int):
    def __new__(cls, foo):
        self = int.__new__(cls)
        self.foo = foo
        return self

    def __getnewargs__(self):
        return (self.foo,)

    def __eq__(self, other):
        return self.foo == other.foo


na = NewArgs(42)
nac = copy.copy(na)
assert isinstance(nac, NewArgs) and nac == na and nac is not na, "getnewargs copy"


# __getnewargs_ex__ supports keyword-only construction.
class NewArgsEx(int):
    def __new__(cls, *, foo):
        self = int.__new__(cls)
        self.foo = foo
        return self

    def __getnewargs_ex__(self):
        return ((), {"foo": self.foo})

    def __eq__(self, other):
        return self.foo == other.foo


nae = NewArgsEx(foo=42)
naec = copy.copy(nae)
assert isinstance(naec, NewArgsEx) and naec == nae, "getnewargs_ex copy"

print("pickle_getnewargs_reconstructs OK")
