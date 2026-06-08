# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "errors"
# case = "metadata_is_read_only"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = "mamba @dataclass field metadata is not a read-only proxy (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "field_defaults.py"
# status = "filled"
# ///
"""dataclasses.field: field metadata is exposed as a read-only mapping; assigning to it raises TypeError"""
from dataclasses import dataclass, field, fields


@dataclass
class Meta:
    i: int = field(metadata={"unit": "px"})


m = fields(Meta)[0].metadata
assert m["unit"] == "px"
_raised = False
try:
    m["new"] = 1
except TypeError:
    _raised = True
assert _raised, "metadata_is_read_only: expected TypeError"
print("metadata_is_read_only OK")
