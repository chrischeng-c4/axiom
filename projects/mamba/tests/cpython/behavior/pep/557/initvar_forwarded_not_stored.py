# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "initvar_forwarded_not_stored"
# subject = "dataclasses.InitVar"
# kind = "semantic"
# xfail = "mamba does not synthesize InitVar forwarding (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "initvar.py"
# status = "filled"
# ///
"""dataclasses.InitVar: an InitVar parameter is passed to __init__ and forwarded to __post_init__ but is not stored as a field"""
from dataclasses import dataclass, fields, InitVar


@dataclass
class Scaled:
    x: int = 0
    factor: InitVar[int] = 1

    def __post_init__(self, factor):
        self.x = self.x * factor


s = Scaled(5, factor=3)
assert s.x == 15
# InitVar does not become a real field: it is absent from fields().
assert [f.name for f in fields(Scaled)] == ["x"]
print("initvar_forwarded_not_stored OK")
