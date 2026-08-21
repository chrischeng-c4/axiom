# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "rfc_4122_variant_constant"
# subject = "uuid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid: rfc_4122_variant_constant (surface)."""
import uuid

assert hasattr(uuid, "RFC_4122")
print("rfc_4122_variant_constant OK")
