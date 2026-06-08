# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "field_metadata_exposed"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = "mamba does not expose field metadata (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "field_defaults.py"
# status = "filled"
# ///
"""dataclasses.field: field(metadata=...) is exposed via fields(C)[i].metadata as a read-only mapping"""
from dataclasses import dataclass, field, fields


@dataclass
class Meta:
    i: int = field(metadata={"unit": "px"})


m = fields(Meta)[0].metadata
assert m["unit"] == "px"
print("field_metadata_exposed OK")
