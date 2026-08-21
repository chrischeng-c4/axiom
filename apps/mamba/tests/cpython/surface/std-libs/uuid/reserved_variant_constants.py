# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "reserved_variant_constants"
# subject = "uuid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid: reserved_variant_constants (surface)."""
import uuid

assert hasattr(uuid, "RESERVED_NCS")
print("reserved_variant_constants OK")
