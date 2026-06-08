# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_namespace_url_is_present"
# subject = "uuid.NAMESPACE_URL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.NAMESPACE_URL: api_namespace_url_is_present (surface)."""
import uuid

assert hasattr(uuid, "NAMESPACE_URL")
print("api_namespace_url_is_present OK")
