# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "namespace_oid_is_uuid"
# subject = "uuid.NAMESPACE_OID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid.NAMESPACE_OID: namespace_oid_is_uuid (surface)."""
import uuid

assert type(uuid.NAMESPACE_OID).__name__ == "UUID"
print("namespace_oid_is_uuid OK")
