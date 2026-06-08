# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "tuple_annotated_fixed_arity"
# subject = "typing.Tuple"
# kind = "semantic"
# xfail = "mamba diverges on the typing generic-alias runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Tuple: a Tuple[int,int]-annotated function returns a real 2-tuple: _divmod2(17,5) unpacks to q==3, r==2"""
import typing
from typing import Tuple


def _divmod2(a: int, b: int) -> Tuple[int, int]:
    return divmod(a, b)


_q, _r = _divmod2(17, 5)
assert _q == 3 and _r == 2, f"divmod = {_q},{_r}"

print("tuple_annotated_fixed_arity OK")
