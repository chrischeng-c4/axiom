# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "post_init_runs_after_init"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = "mamba does not call synthesized __post_init__ (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "initvar.py"
# status = "filled"
# ///
"""dataclasses.field: __post_init__ runs after the generated __init__; an init=False field is still a real field but not an __init__ param"""
from dataclasses import dataclass, field, fields


@dataclass
class Derived:
    x: int
    y: int
    total: int = field(init=False, default=0)

    def __post_init__(self):
        self.total = self.x + self.y


d = Derived(3, 4)
assert d.total == 7
# init=False fields are still real fields, just not __init__ parameters.
assert [f.name for f in fields(Derived)] == ["x", "y", "total"]
print("post_init_runs_after_init OK")
