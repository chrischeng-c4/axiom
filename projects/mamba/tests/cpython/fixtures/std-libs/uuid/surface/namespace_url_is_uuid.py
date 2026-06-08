# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "namespace_url_is_uuid"
# subject = "uuid.NAMESPACE_URL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid.NAMESPACE_URL: namespace_url_is_uuid (surface)."""
import uuid

assert type(uuid.NAMESPACE_URL).__name__ == "UUID"
print("namespace_url_is_uuid OK")
