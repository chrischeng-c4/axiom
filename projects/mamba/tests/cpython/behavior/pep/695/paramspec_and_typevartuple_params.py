# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "paramspec_and_typevartuple_params"
# subject = "typing.ParamSpec"
# kind = "semantic"
# xfail = "**P / *Ts params are not real ParamSpec / TypeVarTuple objects on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.ParamSpec: **P creates a ParamSpec and *Ts creates a TypeVarTuple from the [ ] syntax; both are real typing objects with __infer_variance__ on the ParamSpec"""
from typing import TypeVarTuple, ParamSpec


# **P creates a ParamSpec, *Ts creates a TypeVarTuple.
def make_paramspec[**P]():
    return P


def make_tvt[*Ts]():
    return Ts


p = make_paramspec()
assert isinstance(p, ParamSpec)
assert p.__infer_variance__
ts = make_tvt()
assert isinstance(ts, TypeVarTuple)

print("paramspec_and_typevartuple_params OK")
