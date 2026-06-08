# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "kw_only_sentinel_splits_fields"
# subject = "dataclasses.KW_ONLY"
# kind = "semantic"
# xfail = "mamba does not honor the KW_ONLY sentinel (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "keyword_only.py"
# status = "filled"
# ///
"""dataclasses.KW_ONLY: the KW_ONLY pseudo-field marks every following field as keyword-only"""
from dataclasses import dataclass, KW_ONLY


@dataclass
class Split:
    a: int
    _: KW_ONLY
    b: int
    c: int


s = Split(1, b=2, c=3)
assert (s.a, s.b, s.c) == (1, 2, 3)
print("kw_only_sentinel_splits_fields OK")
