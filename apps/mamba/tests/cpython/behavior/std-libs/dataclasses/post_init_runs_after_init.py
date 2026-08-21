# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "post_init_runs_after_init"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.dataclass: __post_init__ runs after the synthesized __init__ and can derive an init=False field from the constructor-set fields"""
import dataclasses


@dataclasses.dataclass
class WithPost:
    x: int
    y: int
    total: int = dataclasses.field(init=False)

    def __post_init__(self):
        self.total = self.x + self.y


wp = WithPost(3, 4)
assert wp.total == 7, f"__post_init__ total = {wp.total!r}"

print("post_init_runs_after_init OK")
