# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_reserved_ncs_is_present"
# subject = "uuid.RESERVED_NCS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.RESERVED_NCS: api_reserved_ncs_is_present (surface)."""
import uuid

assert hasattr(uuid, "RESERVED_NCS")
print("api_reserved_ncs_is_present OK")
