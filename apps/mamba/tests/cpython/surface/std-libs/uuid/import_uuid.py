# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "import_uuid"
# subject = "uuid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid: import_uuid (surface)."""
import uuid

assert hasattr(uuid, "UUID")
print("import_uuid OK")
