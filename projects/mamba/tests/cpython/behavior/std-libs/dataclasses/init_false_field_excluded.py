# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "init_false_field_excluded"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.field: field(init=False) is excluded from the synthesized __init__ parameter list (its Field.init is False) while init=True fields are included"""
import dataclasses


@dataclasses.dataclass
class WithPost:
    x: int
    y: int
    total: int = dataclasses.field(init=False)

    def __post_init__(self):
        self.total = self.x + self.y


init_fields = [f.name for f in dataclasses.fields(WithPost) if f.init]
assert "total" not in init_fields, "init=False excluded from init params"
assert "x" in init_fields and "y" in init_fields, "init=True fields included"

print("init_false_field_excluded OK")
