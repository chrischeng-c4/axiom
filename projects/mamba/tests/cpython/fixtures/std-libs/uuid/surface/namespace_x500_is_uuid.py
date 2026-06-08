# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "namespace_x500_is_uuid"
# subject = "uuid.NAMESPACE_X500"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid.NAMESPACE_X500: namespace_x500_is_uuid (surface)."""
import uuid

assert type(uuid.NAMESPACE_X500).__name__ == "UUID"
print("namespace_x500_is_uuid OK")
