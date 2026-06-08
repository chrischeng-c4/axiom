# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "local_attribute_roundtrip"
# subject = "threading.local"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.local: an attribute assigned on a threading.local() instance reads back equal in the same thread"""
import threading

_local = threading.local()
_local.x = 42
assert _local.x == 42, f"thread local x = {_local.x!r}"

print("local_attribute_roundtrip OK")
