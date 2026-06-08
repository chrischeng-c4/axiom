# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "kw_only_class_all_fields_keyword"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not honor kw_only=True (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "keyword_only.py"
# status = "filled"
# ///
"""dataclasses.dataclass: @dataclass(kw_only=True) forces every field keyword-only in the generated __init__"""
from dataclasses import dataclass


@dataclass(kw_only=True)
class Conf:
    a: int
    b: int


c = Conf(a=1, b=2)
assert (c.a, c.b) == (1, 2)
print("kw_only_class_all_fields_keyword OK")
