# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_equality_and_hash"
# subject = "inspect.signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: two signatures with identical parameters/annotations are equal and hash-equal; a differing return annotation makes them unequal"""
import inspect

def g1(a, *, b: int) -> float:
    pass

def g2(a, *, b: int) -> float:
    pass

def g3(a, *, b: int) -> int:  # different return annotation
    pass

assert inspect.signature(g1) == inspect.signature(g2), "same sig equal"
assert hash(inspect.signature(g1)) == hash(inspect.signature(g2)), "same sig hash"
assert inspect.signature(g1) != inspect.signature(g3), "return anno differs"

print("signature_equality_and_hash OK")
