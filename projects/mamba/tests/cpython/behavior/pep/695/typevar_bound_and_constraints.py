# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "typevar_bound_and_constraints"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "a bound [ ] param is a plain value (an int), not a typing.TypeVar with __bound__ on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: a [ ] param is a real TypeVar: def with_bound[A: str, B: str|int, C: (int,str)] gives A.__bound__ is str, C.__bound__ is None with __constraints__ (int,str), and __infer_variance__ set"""
from typing import TypeVar


# A type param is a real TypeVar; bound vs constraints are captured separately.
def with_bound[A: str, B: str | int, C: (int, str)]():
    return (A, B, C)


a, b, c = with_bound()
assert isinstance(a, TypeVar)
assert a.__bound__ is str
assert b.__bound__ == str | int
# Constrained params have constraints and a None bound.
assert c.__bound__ is None
assert c.__constraints__ == (int, str)
# Params from [ ] syntax always infer variance (not co/contravariant).
assert a.__infer_variance__ and not a.__covariant__ and not a.__contravariant__

print("typevar_bound_and_constraints OK")
