# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "errors"
# case = "replace_missing_initvar_raises"
# subject = "dataclasses.replace"
# kind = "semantic"
# xfail = "mamba does not synthesize InitVar handling for replace() (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "initvar.py"
# status = "filled"
# ///
"""dataclasses.replace: replace() on an instance with an InitVar must re-supply that InitVar; omitting it raises ValueError/TypeError"""
from dataclasses import dataclass, InitVar, replace


@dataclass
class Doubled:
    x: int
    y: InitVar[int]

    def __post_init__(self, y):
        self.x *= y


c = Doubled(1, 10)
assert c.x == 10
_raised = False
try:
    replace(c, x=3)  # missing InitVar y
except (ValueError, TypeError):
    _raised = True
assert _raised, "replace_missing_initvar_raises: expected ValueError/TypeError"
print("replace_missing_initvar_raises OK")
