# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "uuid3_is_deterministic_md5"
# subject = "uuid.uuid3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid3: uuid3(NAMESPACE_URL, 'http://test.com') is MD5-deterministic (two calls equal) and reports version 3"""
import uuid

a = uuid.uuid3(uuid.NAMESPACE_URL, "http://test.com")
b = uuid.uuid3(uuid.NAMESPACE_URL, "http://test.com")
assert a == b, f"uuid3 not deterministic: {a} vs {b}"
assert a.version == 3, f"uuid3 version = {a.version!r}"
print("uuid3_is_deterministic_md5 OK")
