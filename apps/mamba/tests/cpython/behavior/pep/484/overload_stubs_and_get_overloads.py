# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "overload_stubs_and_get_overloads"
# subject = "typing.overload"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.overload: a function with only @overload stubs raises NotImplementedError when called; once a concrete impl is defined the function works and get_overloads returns its two stubs"""
from typing import get_overloads, overload


# A function with only @overload stubs raises NotImplementedError when called.
@overload
def f(x: int) -> int: ...
@overload
def f(x: str) -> str: ...


try:
    f(1)
    raise AssertionError("expected NotImplementedError")
except NotImplementedError:
    pass


# Once a concrete implementation is defined, the function is usable and its
# overload stubs are retrievable via get_overloads (keyed by the impl).
@overload
def g(x: int) -> int: ...
@overload
def g(x: str) -> str: ...
def g(x):
    return x * 2


assert g(3) == 6
assert g("a") == "aa"
assert len(get_overloads(g)) == 2

print("overload_stubs_and_get_overloads OK")
