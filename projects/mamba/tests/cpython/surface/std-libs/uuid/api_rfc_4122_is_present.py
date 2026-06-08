# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_rfc_4122_is_present"
# subject = "uuid.RFC_4122"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.RFC_4122: api_rfc_4122_is_present (surface)."""
import uuid

assert hasattr(uuid, "RFC_4122")
print("api_rfc_4122_is_present OK")
