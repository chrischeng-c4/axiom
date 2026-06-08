# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "import_dataclasses"
# subject = "dataclasses"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses: import_dataclasses (surface)."""
import dataclasses

assert hasattr(dataclasses, "dataclass")
print("import_dataclasses OK")
