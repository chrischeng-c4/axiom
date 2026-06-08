# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "namespace_dns_is_uuid"
# subject = "uuid.NAMESPACE_DNS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid.NAMESPACE_DNS: namespace_dns_is_uuid (surface)."""
import uuid

assert type(uuid.NAMESPACE_DNS).__name__ == "UUID"
print("namespace_dns_is_uuid OK")
