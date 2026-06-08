# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "match_args_lists_positional_fields"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize __match_args__ (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "keyword_only.py"
# status = "filled"
# ///
"""dataclasses.dataclass: __match_args__ lists positional fields in order; kw_only fields are excluded"""
from dataclasses import dataclass, field, KW_ONLY


@dataclass
class Mixed:
    a: int
    b: int = field(kw_only=True)


@dataclass(kw_only=True)
class Conf:
    a: int
    b: int


@dataclass
class Plain:
    x: int
    y: int


assert Mixed(1, b=2).__match_args__ == ("a",)
assert Conf(a=1, b=2).__match_args__ == ()  # all kw_only -> empty
assert Plain(1, 2).__match_args__ == ("x", "y")
print("match_args_lists_positional_fields OK")
