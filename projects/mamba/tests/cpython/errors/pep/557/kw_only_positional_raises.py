# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "errors"
# case = "kw_only_positional_raises"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not enforce keyword-only synthesized __init__ params (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "keyword_only.py"
# status = "filled"
# ///
"""dataclasses.dataclass: passing positional args to a kw_only=True dataclass raises TypeError"""
from dataclasses import dataclass


@dataclass(kw_only=True)
class Conf:
    a: int
    b: int


_raised = False
try:
    Conf(1, 2)  # positional not allowed
except TypeError:
    _raised = True
assert _raised, "kw_only_positional_raises: expected TypeError"
print("kw_only_positional_raises OK")
