# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "typevar_references_earlier_param"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "ordered.__type_params__ returns None and params are not TypeVars on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: a later type param may reference an earlier one in its bound: def ordered[S, T: Sequence[S]] has T.__bound__ == Sequence[S]"""
from typing import Sequence


# A later type param may reference an earlier one in its bound.
def ordered[S, T: Sequence[S]]():
    pass


s_var, t_var = ordered.__type_params__
assert t_var.__bound__ == Sequence[s_var]

print("typevar_references_earlier_param OK")
